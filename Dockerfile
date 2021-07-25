# https://github.com/awslabs/aws-lambda-rust-runtime/issues/17#issuecomment-453635842
FROM lambci/lambda:build-provided
RUN curl https://sh.rustup.rs -sSf | /bin/sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup install stable
RUN mkdir /code
WORKDIR /code