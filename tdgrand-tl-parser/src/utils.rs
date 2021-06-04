// Copyright 2020 - developers of the `grammers` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains several free-standing utility functions.

use crc32fast::Hasher;

/// Infers the identifier for a definition.
pub(crate) fn infer_id(definition: &str) -> u32 {
    let mut representation = definition
        .replace(":bytes ", ": string")
        .replace("?bytes ", "? string")
        .replace("<", " ")
        .replace(">", "")
        .replace("{", "")
        .replace("}", "");

    // Remove r" \w+:flags\.\d+\?true"
    while let Some(pos) = representation.find("?true") {
        let space = representation[..pos].rfind(' ').unwrap_or(0);
        representation.replace_range(space..pos + "?true".len(), "");
    }

    let mut hasher = Hasher::new();
    hasher.update(representation.as_bytes());
    hasher.finalize() // crc32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_infer_id() {
        // Note the type `bytes`
        let def = "rpc_answer_dropped msg_id:long seq_no:int bytes:int = RpcDropAnswer";
        assert_eq!(infer_id(def), 0xa43ad8b7);

        // Note the use of angle brackets
        let def = "msgs_ack msg_ids:Vector<long> = MsgsAck";
        assert_eq!(infer_id(def), 0x62d6b459);

        // Note the use of curly brackets
        let def = "invokeAfterMsg {X:Type} msg_id:long query:!X = X";
        assert_eq!(infer_id(def), 0xcb9f372d);

        // Note the use of `true` flags
        let def = "inputMessagesFilterPhoneCalls flags:# missed:flags.0?true = MessagesFilter";
        assert_eq!(infer_id(def), 0x80c99768);
    }
}
