docker run --rm -v `pwd`:/usr/src/myapp manager_builder cargo build --target-dir="target_docker"
docker build -t gcr.io/mines-mines/manager .
# docker push gcr.io/mines-mines/manager