use std::collections::HashMap;

use ryuji_rust::{ Renderer, FileExtension, Vars, VarValue };

fn main() {
  let renderer: Renderer = Renderer::new("templates".to_string(), "components".to_string(), FileExtension::new(".html".to_string()).unwrap());
  let mut vars: Vars = HashMap::from([
    ("next_post".to_string(), VarValue::HashMap(HashMap::from([
      ("title".to_string(), VarValue::String("Asdf".to_string())),
      ("slug".to_string(), VarValue::String("asdf".to_string())),
    ]))),
    ("post".to_string(), VarValue::HashMap(HashMap::from([
      ("title".to_string(), VarValue::String("Title 123".to_string())),
      ("date".to_string(), VarValue::String("24/01/1999".to_string())),
      ("author".to_string(), VarValue::String("Bush".to_string())),
      ("html".to_string(), VarValue::String("<p>Lorem <b>Ipsum</b></p>".to_string())),
      ("slug".to_string(), VarValue::String("title-123".to_string())),
      ("thanks".to_string(), VarValue::Vec(vec![
        VarValue::String("Saki".to_string()),
        VarValue::String("Joel".to_string()),
        VarValue::String("ryuji_rust".to_string()),
        VarValue::String("Makoto".to_string()),
        VarValue::String("Billy".to_string()),
      ])),
      ("tags_exist".to_string(), VarValue::Bool(true)),
      ("tags".to_string(), VarValue::Vec(vec![
        VarValue::String("testing".to_string()),
        VarValue::String("bloat".to_string()),
        VarValue::String("example".to_string()),
        VarValue::String("crab".to_string()),
      ])),
    ]))),
    ("disclaimer".to_string(), VarValue::Bool(false)),
    ("author_expected".to_string(), VarValue::Bool(true)),
  ]);
  let rendered: String = renderer.render_template("post".to_string(), &mut vars, None).unwrap();
  println!("{}", rendered);
  //assert_eq!(rendered, "15\n<h1>title: abc</h1>\n<div>\n  <p>false</p>\n  a\n  b\n  c\n</div>\n<img/>&lt;img/&gt;");
}
