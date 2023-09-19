# tdlib-rs

A Rust wrapper around the Telegram Database library. It includes a generator to automatically generate the types and functions from the TDLib's [Type Language](https://core.telegram.org/mtproto/TL) file.

It's mainly created for using it in the [Telegrand](https://github.com/melix99/telegrand) client, but it should work also for any other Rust project.

Current supported TDLib version: [1.8.18](https://github.com/tdlib/td/commit/100ca72b9179366d91089fd929f6c210f697031a).

## Credits

- [grammers](https://github.com/Lonami/grammers): the `tdlib-tl-gen` and `tdlib-tl-parser` projects are forks of the `grammers-tl-gen` and `grammers-tl-parser` projects.
- [rust-tdlib](https://github.com/aCLr/rust-tdlib): for inspiration about some client code.
