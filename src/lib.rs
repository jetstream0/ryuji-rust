//! Ryuji-Rust is an implementation of the Ryuji templating language in Rust.
pub mod ryuji;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn file_extension_coerce() {
    assert!(ryuji::FileExtension::new("adsf".to_string()).is_err());
    assert!(ryuji::FileExtension::new("ad.sf".to_string()).is_err());
    assert!(ryuji::FileExtension::new(".png".to_string()).is_ok());
    assert_eq!(ryuji::FileExtension::try_from(".py".to_string()).unwrap(), ryuji::FileExtension::new(".py".to_string()).unwrap());
    assert_eq!(".py".to_string(), String::from(ryuji::FileExtension::new(".py".to_string()).unwrap()));
  }

  #[test]
  fn concat_path_test() {
    assert_eq!(ryuji::Renderer::concat_path(&"abc/".to_string(), &"/tree.html".to_string()), "abc/tree.html".to_string());
    assert_eq!(ryuji::Renderer::concat_path(&"/abc/".to_string(), &"/tree.html".to_string()), "/abc/tree.html".to_string());
    assert_eq!(ryuji::Renderer::concat_path(&"abc/".to_string(), &"tree.html".to_string()), "abc/tree.html".to_string());
    assert_eq!(ryuji::Renderer::concat_path(&"abc".to_string(), &"/tree.html".to_string()), "abc/tree.html".to_string());
    assert_eq!(ryuji::Renderer::concat_path(&"abc/def".to_string(), &"tree.html".to_string()), "abc/def/tree.html".to_string());
  }

  #[test]
  fn sanitize_test() {
    assert_eq!(ryuji::Renderer::sanitize(&"asdf".to_string()), "asdf".to_string());
    assert_eq!(ryuji::Renderer::sanitize(&"<script>a</script>".to_string()), "&lt;script&gt;a&lt;/script&gt;".to_string());
  }

  #[test]
  fn var_name_legality_test() {
    assert!(ryuji::Renderer::check_var_name_legality(&"asdf".to_string(), true).is_ok());
    assert!(ryuji::Renderer::check_var_name_legality(&"random/abc".to_string(), true).is_ok());
    assert!(ryuji::Renderer::check_var_name_legality(&"cheese.burger.property40".to_string(), true).is_ok());
    assert!(ryuji::Renderer::check_var_name_legality(&"Dave_Davidson.drunkness.intensity".to_string(), true).is_ok());
    assert!(ryuji::Renderer::check_var_name_legality(&"Dave_Davidson.drunkness.intensity".to_string(), false).is_err());
    assert!(ryuji::Renderer::check_var_name_legality(&"+23;.'wow'".to_string(), true).is_err());
    assert!(ryuji::Renderer::check_var_name_legality(&"test ".to_string(), true).is_err());
  }

  #[test]
  fn find_syntax_matches_test() {
    assert_eq!(ryuji::Renderer::find_syntax_matches(&"[[ test.e ]]\n[[]]\nyay [[ if:yay ]]".to_string()), vec![
      ryuji::SyntaxMatch {
        index: 0,
        content: "[[ test.e ]]".to_string(),
      },
      ryuji::SyntaxMatch {
        index: 22,
        content: "[[ if:yay ]]".to_string(),
      },
    ]);
    assert_eq!(ryuji::Renderer::find_syntax_matches(&"lorem\n[[ \na ]]\nhello [[ na=me ]]".to_string()), vec![]);
  }

  #[test]
  fn variable_test() {
    let renderer: ryuji::Renderer = ryuji::Renderer::new("templates".to_string(), "components".to_string(), ryuji::FileExtension::new(".html".to_string()).unwrap());
    let mut vars: ryuji::Vars = std::collections::HashMap::from([
      ("a".to_string(), ryuji::VarValue::U32(15)),
      ("b".to_string(), ryuji::VarValue::HashMap(std::collections::HashMap::from([
        ("c".to_string(), ryuji::VarValue::String("abc".to_string())),
      ]))),
      ("d".to_string(), ryuji::VarValue::Bool(false)),
      ("testing_123".to_string(), ryuji::VarValue::String("a\nb\nc".to_string())),
      ("img".to_string(), ryuji::VarValue::String("<img/>".to_string())),
    ]);
    let rendered: String = renderer.render("[[ a ]]\n<h1>title: [[ b.c ]]</h1>\n<div>\n  <p>[[ d ]]</p>\n  [[ testing_123 ]]\n</div>\n[[ html:img ]][[ img ]]".to_string(), &mut vars, None).unwrap();
    assert_eq!(rendered, "15\n<h1>title: abc</h1>\n<div>\n  <p>false</p>\n  a\n  b\n  c\n</div>\n<img/>&lt;img/&gt;");
  }

  #[test]
  fn if_test() {
    let renderer: ryuji::Renderer = ryuji::Renderer::new("templates".to_string(), "components".to_string(), ryuji::FileExtension::new(".html".to_string()).unwrap());
    let mut vars: ryuji::Vars = std::collections::HashMap::from([
      ("koalas_list".to_string(), ryuji::VarValue::Vec(Vec::new())),
      ("oak".to_string(), ryuji::VarValue::HashMap(std::collections::HashMap::from([
        ("is_tree".to_string(), ryuji::VarValue::Bool(true)),
        ("is_not_tree".to_string(), ryuji::VarValue::Bool(false)),
      ]))),
      ("pine".to_string(), ryuji::VarValue::HashMap(std::collections::HashMap::from([
        ("is_tree".to_string(), ryuji::VarValue::Bool(true)),
        ("is_not_tree".to_string(), ryuji::VarValue::Bool(false)),
      ]))),
      ("dave".to_string(), ryuji::VarValue::HashMap(std::collections::HashMap::from([
        ("is_tree".to_string(), ryuji::VarValue::Bool(false)),
        ("is_not_tree".to_string(), ryuji::VarValue::Bool(true)),
      ]))),
    ]);
    let rendered: String = renderer.render("[[ if:koalas_list ]]<p>We have a list of koalas</p>\n[[ endif ]]<p>Dave is [[ if:dave.is_not_tree ]]not a tree[[ endif ]][[ if:dave.is_tree ]]a tree[[ endif ]]</p>\n[[ if:pine.is_tree:oak.is_tree ]]<b>Oak and pine are both trees.</b>[[ endif ]]\n[[ if:dave.is_tree:!oak.is_tree ]]<i>But Dave and Oak are different. One of them is a tree, and one of them is not a tree.</i>[[ endif ]]".to_string(), &mut vars, None).unwrap();
    assert_eq!(rendered, "<p>Dave is not a tree</p>\n<b>Oak and pine are both trees.</b>\n<i>But Dave and Oak are different. One of them is a tree, and one of them is not a tree.</i>");
  }

  #[test]
  fn for_loop_test() {
    //am lazy so these tests are copied from typescript ryuji's tests, more or less
    let renderer: ryuji::Renderer = ryuji::Renderer::new("templates".to_string(), "components".to_string(), ryuji::FileExtension::new(".html".to_string()).unwrap());

    //empty for loop test
    let mut vars_empty: ryuji::Vars = std::collections::HashMap::from([
      ("loop_over".to_string(), ryuji::VarValue::Vec(Vec::new())),
    ]);
    let rendered_empty: String = renderer.render("<ul>\n  [[ for:loop_over ]]a[[ endfor ]]\n</ul>\n<p>[[ for:loop_over:item ]][[ endfor ]]</p>".to_string(), &mut vars_empty, None).unwrap();
    assert_eq!(rendered_empty, "<ul>\n  \n</ul>\n<p></p>");

    //for loop with more vars and if statement test
    let mut vars_max: ryuji::Vars = std::collections::HashMap::from([
      ("trees".to_string(), ryuji::VarValue::Vec(vec![
        ryuji::VarValue::String("mango".to_string()),
        ryuji::VarValue::String("oak".to_string()),
        ryuji::VarValue::String("redwood".to_string()),
        ryuji::VarValue::String("palm".to_string()),
      ])),
    ]);
    let rendered_if: String = renderer.render("[[ for:trees:tree:index_var:max_var ]][[ index_var ]]/[[ max_var ]] [[ tree ]][[ if:index_var:!max_var ]] [[ endif ]][[ endfor ]]".to_string(), &mut vars_max, None).unwrap();
    assert_eq!(rendered_if, "0/3 mango 1/3 oak 2/3 redwood 3/3 palm");

    //another for loop with if statement test
    let mut vars_if2: ryuji::Vars = std::collections::HashMap::from([
      ("letters".to_string(), ryuji::VarValue::Vec(vec![
        ryuji::VarValue::HashMap(std::collections::HashMap::from([
          ("letter".to_string(), ryuji::VarValue::String("a".to_string())),
          ("show".to_string(), ryuji::VarValue::Bool(true)),
        ])),
        ryuji::VarValue::HashMap(std::collections::HashMap::from([
          ("letter".to_string(), ryuji::VarValue::String("b".to_string())),
          ("show".to_string(), ryuji::VarValue::Bool(false)),
        ])),
        ryuji::VarValue::HashMap(std::collections::HashMap::from([
          ("letter".to_string(), ryuji::VarValue::String("c".to_string())),
          ("show".to_string(), ryuji::VarValue::Bool(true)),
        ])),
        ryuji::VarValue::HashMap(std::collections::HashMap::from([
          ("letter".to_string(), ryuji::VarValue::String("d".to_string())),
          ("show".to_string(), ryuji::VarValue::Bool(false)),
        ])),
      ])),
    ]);
    let rendered_if2: String = renderer.render("[[ for:letters:letter ]][[ if:letter.show ]]<p>[[ letter.letter ]]</p>[[ endif ]][[ endfor ]]".to_string(), &mut vars_if2, None).unwrap();
    assert_eq!(rendered_if2, "<p>a</p><p>c</p>");

    //nested for loop test
    let mut vars_nested: ryuji::Vars = std::collections::HashMap::from([
      ("numbers".to_string(), ryuji::VarValue::Vec(vec![
        ryuji::VarValue::U32(1),
        ryuji::VarValue::U32(2),
        ryuji::VarValue::U32(3),
      ])),
    ]);
    let rendered_nested: String = renderer.render("[[ for:numbers:i ]].[[ i ]].[[ for:numbers:j ]][[ j ]][[ endfor ]][[ endfor ]]".to_string(), &mut vars_nested, None).unwrap();
    assert_eq!(rendered_nested, ".1.123.2.123.3.123");
  }
}
