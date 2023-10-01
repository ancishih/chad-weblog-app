FROM messense/rust-musl-cross:x86_64-musl-amd64 as chef
ENV SQLX_OFFLINE=true
RUN cargo install cargo-chef
WORKDIR /chad-weblog-app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /chad-weblog-app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl


FROM scratch
COPY --from=builder /chad-weblog-app/target/x86_64-unknown-linux-musl/release/chad-weblog-app /chad-weblog-app
ENTRYPOINT [ "/chad-weblog-app" ]
EXPOSE 5599
