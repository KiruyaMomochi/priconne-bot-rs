# [Choice] Debian OS version (use bullseye on local arm64/Apple Silicon): buster, bullseye
ARG VARIANT="latest"
FROM mcr.microsoft.com/vscode/devcontainers/rust:${VARIANT}

# Install mongo shell
# Include lld linker to improve build times either by using environment variable
# RUSTFLAGS="-C link-arg=-fuse-ld=lld" or with Cargo's configuration file (i.e see .cargo/config.toml).
RUN sudo apt-get install gnupg \
   && curl -fsSL https://pgp.mongodb.com/server-6.0.asc | \
   sudo gpg -o /usr/share/keyrings/mongodb-server-6.0.gpg \
   --dearmor \
   && echo "deb [ signed-by=/usr/share/keyrings/mongodb-server-6.0.gpg] http://repo.mongodb.org/apt/debian bullseye/mongodb-org/6.0 main" | sudo tee /etc/apt/sources.list.d/mongodb-org-6.0.list \
   && apt-get update && export DEBIAN_FRONTEND=noninteractive \
   && apt-get -y install clang lld mongodb-mongosh \
   && apt-get autoremove -y && apt-get clean -y
