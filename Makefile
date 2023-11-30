REPO = oostvoort/keiko

VERSION ?= v0.0.9
DOJO_VERSION = v0.3.10

# Example: make docker-build version=v1.1.0
build:
	docker build -t $(REPO):$(VERSION) -t $(REPO):latest --build-arg DOJO_VERSION=$(DOJO_VERSION) .

# Example: make docker-run version=v1.2.9
run:
	docker run -p 3000:3000 -p 5050:5050 -p 8080:8080 $(REPO):$(VERSION)

shell:
	docker run -it --rm --name temp-container oostvoort/keiko:latest bash

# Example: make docker-push version=v1.1.0
push:
	docker push $(REPO):latest
	docker push $(REPO):$(VERSION)

tag:
	# Create a new tag
	git tag $(VERSION)

	# Push the tag to the remote repository
	git push origin $(VERSION)