.PHONY: deploy stack-outputs

deploy:
	sam build
	sam deploy
	$(MAKE) foo

foo:
	aws cloudformation describe-stacks	\
	--stack-name complex-sfn-test	\
	--query "Stacks[0].Outputs"	\
	| jq 'reduce .[] as $$item ({}; .[$$item.OutputKey] = $$item.OutputValue)'	\
	| jq -r 'keys[] as $$k | "\($$k)=\(.[$$k])"'	\
	> .outputs.env
