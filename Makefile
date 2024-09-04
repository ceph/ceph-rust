DOCKER_CI_IMAGE = ceph-rust-ci

build:
	cargo build
fmt:
	cargo fmt
test:
	cargo test

test-docker: .build-docker
	docker run --rm -it -v $(CURDIR):/ceph-rust $(DOCKER_CI_IMAGE)

shell-docker: .build-docker
	docker run --rm -it -v $(CURDIR):/ceph-rust $(DOCKER_CI_IMAGE) bash
	# Now you can run
	# . /setup-micro-osd.sh
	# cargo build

.build-docker:
	docker build -t $(DOCKER_CI_IMAGE) .
	@docker inspect -f '{{.Id}}' $(DOCKER_CI_IMAGE) > .build-docker
