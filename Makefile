build-NotifyCostToSlack:
	docker build . -t lambda_builder
	docker run --rm -v ${PWD}:/code -v ${HOME}/.cargo/registry:/root/.cargo/registry -v ${HOME}/.cargo/git:/root/.cargo/git lambda_builder cargo build --release
	cp ./target/release/aws_cost_notification ./bootstrap
	zip -j lambda.zip bootstrap
	rm bootstrap

build:
	sam build

deploy:
	sam deploy

all: build deploy

