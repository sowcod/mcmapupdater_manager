use crate::config::{Config, ConfigDocker, StrVec};
use crate::config::ConfigDockerCreds::{Gce, Token};
use crate::CommonResult;
use reqwest::Client;
use futures_util::StreamExt;
use shiplift::{
    Docker,
    PullOptions,
    ContainerOptions,
    RegistryAuth,
    tty::TtyChunk,
};

#[derive(serde::Deserialize)]
struct GceAccessToken {
    pub access_token: String,
    pub expires_in: u32,
    pub token_type: String,
}

pub struct Manager<'a> {
    config: Config,
    docker: &'a Docker,
}

impl<'a> Manager<'a> {
    pub fn new(config: Config, docker: &'a Docker) -> Manager {
        return Manager {
            config,
            docker: docker
        }
    }

    pub async fn run_genmap_all(&self) -> CommonResult<()> {
        // println!("run_test");
        // let result = self.run_dockerjob(&ConfigDocker {
        //     image: String::from("eeacms/rsync:latest"),
        //     cred: None,
        //     volumes: vec![],
        // }, vec!["rsync"]).await?;
        // println!("result:{}", result);
        println!("run_map_copy");
        let result = self.run_map_copy().await?;
        if result != 0 {
            println!("rsync exit code {}", result);
            return Ok(());
        }
        // TODO: オプションで実行箇所を指定できるように。
        println!("run_genmap");
        let result = self.run_genmap().await?;
        if result != 0 {
            println!("rsync exit code {}", result);
            return Ok(());
        }
        println!("run_tiler");
        let result = self.run_tiler().await?;
        if result != 0 {
            println!("rsync exit code {}", result);
            return Ok(());
        }
        println!("run_upload");
        let result = self.run_upload().await?;
        if result != 0 {
            println!("rsync exit code {}", result);
            return Ok(());
        }
        Ok(())
    }

    async fn run_map_copy(&self) -> CommonResult<u64> {
        // gamevmからマップデータをコピー
        let map_copy = &self.config.map_copy;
        // let rsync_args = &map_copy.rsync_args;
        // let command = 
        //     // TODO: 一つの文字列になってしまっているようなので、配列に分割必要。
        //     format!(r#"rsync -ahv --delete -e "ssh -p {port} -i {key} -o strictHostKeyChecking=no" {src} {dest}"#,
        //         port = rsync_args.port,
        //         key = rsync_args.key,
        //         src = rsync_args.copy_from,
        //         dest = rsync_args.copy_to,
        //     );
        self.run_dockerjob(&map_copy.docker, map_copy.args.to_strvec()).await
    }

    async fn run_genmap(&self) -> CommonResult<u64> {
        // マップデータから2Dマップを生成
        let genmap = &self.config.genmap;
        self.run_dockerjob(&genmap.docker, genmap.args.to_strvec()).await
    }

    async fn run_tiler(&self) -> CommonResult<u64> {
        // 2Dマップからズームアウトタイル画像を生成
        let tiler = &self.config.tiler;
        // let command = 
        //     format!(r#"-a {max} -i 1 {output}"#,
        //         max = tiler.args.max,
        //         output = tiler.args.output,
        //     );
        self.run_dockerjob(&self.config.tiler.docker, tiler.args.to_strvec()).await
    }

    async fn run_upload(&self) -> CommonResult<u64> {
        // GCSにタイル画像をアップロード
        let upload = &self.config.upload;
        self.run_dockerjob(&upload.docker, upload.args.to_strvec()).await
    }

    async fn run_dockerjob(&self, docker_param: &ConfigDocker, command: Vec<&str>) -> CommonResult<u64> {
        let image = &docker_param.image;
        println!("Pull image: {}", image);
        match self.pull_image(image, &docker_param.cred).await {
            Err(e) => {
                println!("pull error {:?}", e);
                return Err(e);
            },
            _ => (),
        }

        let container_op = ContainerOptions::builder(image)
            .volumes(docker_param.volumes.to_strvec())
            .cmd(command.clone())
            .build();
        let container_info = self.docker.containers().create(&container_op).await?;
        println!("container created");
        let id = container_info.id;
        let containers = self.docker.containers();
        let container = containers.get(&id);
        println!("container built");

        match container.start().await {
            Err(e) => {
                println!("container start error: {:?}", e);
                return Err(e.into());
            },
            Ok(_) => ()
        }
        println!("container start");
        let (mut reader, _writer) = container.attach().await?.split();
        while let Some(tty_result) = reader.next().await {
            match tty_result {
                Ok(chunk) => print_chunk(chunk),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return Err(Box::new(e));
                }
            }
        }
        let exit_code = container.inspect().await?.state.exit_code;
        container.delete().await?;

        Ok(exit_code)
    }

    async fn get_gcetoken(&self) -> CommonResult<String> {
        println!("Get GceToken");
        let client = Client::new();
        let res1 = client.get("http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token")
            .header("Metadata-Flavor", "Google")
            .send()
            .await;
        println!("res1: {:?}", res1); // なぜかここに到達せずに終了する。
        let res = match res1 {
            Ok(res) => res,
            Err(e) => {
                println!("Getting token request error: {:?}", e);
                return Err(Box::new(e));
            }
        };
        let json = res.json::<GceAccessToken>().await;
        match json {
            Ok(gce_token) => return Ok(gce_token.access_token),
            Err(e) => {
                println!("Parsing token error: {:?}", e);
                return Err(Box::new(e));
            }
        }

        // let gce_token = client.get("http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token")
        //     .header("Metadata-Flavor", "Google")
        //     .send()
        //     .await
        //     .or_else(|e| Err(e))?
        //     .json::<GceAccessToken>()
        //     .await
        //     .or_else(|e| Err(e))?;
        //
        // Ok(gce_token.access_token)
    }

    async fn get_auth(&self, cred: &Option<String>) -> CommonResult<Option<RegistryAuth>> {
        // cred is None -> None
        // creds[cred] is None -> None
        if let Some(cred_name) = cred {
            println!("cred name: {}", cred_name);
        }
        let credtype = (|| Some(self.config.creds.get(cred.as_ref()?)?))();
        if let Some(credtype) = credtype {
            println!("cred type: {:?}", credtype);
        }

        let auth = match credtype {
            Some(cred) => match cred {
                Gce => 
                    match self.get_gcetoken().await {
                        Ok(token) => 
                            RegistryAuth::builder()
                                .username("oauth2accesstoken")
                                .password(token)
                                .build()
                        ,
                        Err(e) => {
                            println!("gce_token error: {:?}", e);
                            return Err(e);
                        }
                    },
                Token{ token } => RegistryAuth::builder()
                        .username("oauth2accesstoken")
                        .password(token)
                        .build()
            },
            None => {
                println!("cred None");
                return Ok(None);
            }
        };

        Ok(Some(auth))
    }

    async fn pull_image<T: Into<String>>(&self, image: T, cred: &Option<String>) -> CommonResult<()> {
        let pull_op = match self.get_auth(cred).await? {
            Some(auth) => PullOptions::builder()
                .auth(auth)
                .image(image)
                .build(),
            None => PullOptions::builder()
                .image(image)
                .build(),
        };
        let mut stream = self.docker.images().pull(&pull_op);
        while let Some(pull_result) = stream.next().await {
            match pull_result {
                Ok(output) => println!("{:?}", output),
                Err(e) => {
                    eprintln!("Pull image error: {}", e);
                    return Err(Box::new(e));
                }
            }
        }
        
        Ok(())
    }

}

fn print_chunk(chunk: TtyChunk) {
    match chunk {
        TtyChunk::StdOut(bytes) => print!("Stdout: {}", std::str::from_utf8(&bytes).unwrap()),
        TtyChunk::StdErr(bytes) => eprint!("Stdout: {}", std::str::from_utf8(&bytes).unwrap()),
        TtyChunk::StdIn(_) => unreachable!(),
    }
}