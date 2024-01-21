# gitlab-components-docs

A dead simple auto-docs generator for [Gitlab CI/CD components][gitlab-components].

## Beta Notice :warning:

As of this writing, GitLab CI/CD components are a Beta feature so expect changes :)

[gitlab-components]: https://docs.gitlab.com/ee/ci/components/

## Installation

If you want to run it directly, you can use cargo to build and install it on your workstation.

```shell
cargo install --git https://github.com/raskyld/gitlab-components-docs --tag 0.2.0
```

I recommend you, though, that you run the CLI using an OCI runtime such as Docker.

## Usage

This CLI comes with sensible default, all you have to provide is the name and description of
your catalog.

```
docker run --rm -v "$(pwd):/gitlab" -u "$(id -u)" ghcr.io/raskyld/gitlab-components-docs:0.2.0 \
    -n <YOUR_CATALOG_NAME> \
    -d "<YOUR_CATALOG_DESCRIPTION>"
```

### Custom Template

This CLI uses the awesome [Tera](https://keats.github.io/tera/) template engine.

It will try to lookup for a file named `README.md.tera` at the root of your repo,
allowing you to customise the output. If none is found, use the default template
(that you can find [there](src/templates.rs)).
