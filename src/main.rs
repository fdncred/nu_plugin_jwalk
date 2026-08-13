use nu_plugin::{MsgPackSerializer, serve_plugin};
use nu_plugin_jwalk::JWalkPlugin;

fn main() {
    serve_plugin(&JWalkPlugin, MsgPackSerializer);
}
