FROM ekidd/rust-musl-builder:stable as builder

RUN USER=root cargo new --bin juejin-automatic-sign-draw
WORKDIR ./juejin-automatic-sign-draw
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

RUN rm ./target/x86_64-unknown-linux-musl/release/deps/juejin_automatic_sign_draw*

COPY ./src ./src

COPY ./config ./config

RUN cargo build --release


FROM alpine:latest

ARG APP=/usr/src/app

EXPOSE 8000

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN addgroup -S $APP_USER \
    && adduser -S -g $APP_USER $APP_USER

RUN apk update \
    && apk add --no-cache ca-certificates tzdata \
    && rm -rf /var/cache/apk/*

COPY --from=builder /home/rust/src/juejin-automatic-sign-draw/config ${APP}/config

COPY --from=builder /home/rust/src/juejin-automatic-sign-draw/target/x86_64-unknown-linux-musl/release/juejin-automatic-sign-draw ${APP}/juejin-automatic-sign-draw

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./juejin-automatic-sign-draw"]
