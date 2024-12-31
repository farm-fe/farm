use farmfe_core::{
  config::comments::CommentsConfig,
  swc_common::{
    comments::{Comment, CommentKind, SingleThreadedComments},
    BytePos,
  },
};

/// minify comments, the rule is same as swc, see https://github.com/swc-project/swc/blob/main/crates/swc_compiler_base/src/lib.rs
pub fn minify_comments(comments: &SingleThreadedComments, config: &CommentsConfig) {
  match config {
    // preserve all comments
    CommentsConfig::Bool(true) => {}
    CommentsConfig::Bool(false) => {
      let (mut l, mut t) = comments.borrow_all_mut();
      l.clear();
      t.clear();
    }
    CommentsConfig::License => {
      let preserve_excl = |_: &BytePos, vc: &mut Vec<Comment>| -> bool {
        // Preserve license comments.
        //
        // See https://github.com/terser/terser/blob/798135e04baddd94fea403cfaab4ba8b22b1b524/lib/output.js#L175-L181
        vc.retain(|c: &Comment| {
          c.text.contains("@lic")
            || c.text.contains("@preserve")
            || c.text.contains("@copyright")
            || c.text.contains("@cc_on")
            || (c.kind == CommentKind::Block && c.text.starts_with('!'))
        });
        !vc.is_empty()
      };
      let (mut l, mut t) = comments.borrow_all_mut();

      l.retain(preserve_excl);
      t.retain(preserve_excl);
    }
  }
}
