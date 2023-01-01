.PHONY: build rename

build:
	cargo lambda build --release --arm64 --output-format zip
	rm -rf bin
	cp -R target/lambda bin
	$(MAKE) rename


# We have to rename files as I cannot have multiple bootstrap.zip files.
# The serverless framework does not support multiple bootstrap.zip files in `artifact` property.
MYDIR = bin
rename: $(MYDIR)/*/*
	for file in $^ ; do \
		dir=$$(dirname $$file); \
		package_file=$$(basename $$file); \
		new_package_file_name=$$(basename $$dir); \
		mv $$dir/$$package_file $$dir/$${new_package_file_name}.zip; \
	done
