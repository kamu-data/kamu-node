ifeq (, $(shell which podman 2>/dev/null))
RUNTIME = docker
else
RUNTIME = podman
endif

PLANTUML_IMG = "docker.io/plantuml/plantuml:1.2022.14"
PLANTUML = $(RUNTIME) run --rm -v $(PWD):/opt/workdir --workdir /opt/workdir $(PLANTUML_IMG)

DIAGRAMS_SRC = $(wildcard */*.puml)
DIAGRAMS = $(patsubst %.puml,%.svg,$(DIAGRAMS_SRC))


all: $(DIAGRAMS)

%.svg: %.puml
	$(PLANTUML) -tsvg $^
