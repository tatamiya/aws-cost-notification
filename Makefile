# Build Docker image to build the lambda function.
# If you build the function for the first time,
# you have to execute this command at first.
build-image:
	docker build . -t lambda_builder

# Build command used in building the SAM template.
# Usually you do not need to directly execute this command.
build-NotifyCostToSlack:
	docker run --rm -v ${PWD}:/code -v ${HOME}/.cargo/registry:/root/.cargo/registry -v ${HOME}/.cargo/git:/root/.cargo/git lambda_builder cargo build --release
	cp ./target/release/bootstrap ${ARTIFACTS_DIR}

# Build the Lambda function and the SAM template.
# Build options should be described in samconfig.toml.
build:
	sam build

# Deploy the Lambda function to AWS.
# Deploy options should be described in samconfig.toml.
deploy:
	sam deploy
