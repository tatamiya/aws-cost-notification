build-image:
	docker build . -t lambda_builder

build-NotifyCostToSlack:
	docker run --rm -v ${PWD}:/code -v ${HOME}/.cargo/registry:/root/.cargo/registry -v ${HOME}/.cargo/git:/root/.cargo/git lambda_builder cargo build --release
	cp ./target/release/bootstrap ${ARTIFACTS_DIR}

build:
	sam build

deploy:
	sam deploy

all: build deploy

