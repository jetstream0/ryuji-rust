use std::collections::{ HashMap, VecDeque };
use std::fmt;
use std::fs;
use std::convert::TryFrom;

#[cfg(feature = "hashmap_json")]
use serde::Serialize;

#[derive(Debug)]
pub enum ErrorKind {
  InvalidFileExtension,
  IllegalVarName(String),
  VarNotFound(String),
  BadArgument(String),
  MissingEndFor,
  MissingEndIf,
  RecursionTooDeep,
}

impl fmt::Display for ErrorKind {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ErrorKind::InvalidFileExtension => write!(f, "Invalid file extension (must start with '.')"),
      ErrorKind::IllegalVarName(var_name) => write!(f, "Illegal variable name: '{}'", var_name),
      ErrorKind::VarNotFound(var_name) => write!(f, "Variable '{}' not found", var_name),
      ErrorKind::BadArgument(missing_message) => write!(f, "{}", missing_message),
      ErrorKind::MissingEndFor => write!(f, "`for:` statement missing `[[ endfor ]]`"),
      ErrorKind::MissingEndIf => write!(f, "`if:` statement missing `[[ endif ]]`"),
      ErrorKind::RecursionTooDeep => write!(f, "`component:` statement recursion too deep (>5)"),
    }
  }
}

#[derive(Debug, PartialEq)]
pub struct FileExtension {
  file_extension: String,
}

//todo: add errors
impl FileExtension {
  pub fn new(file_extension: String) -> Result<Self, ErrorKind> {
    if file_extension.starts_with(".") {
      Ok(FileExtension {
        file_extension,
      })
    } else {
      Err(ErrorKind::InvalidFileExtension)
    }
  }

  pub fn get_string_ref(&self) -> &String {
    &self.file_extension
  }
}

impl TryFrom<String> for FileExtension {
  type Error = ErrorKind;

  fn try_from(a: String) -> Result<Self, ErrorKind> {
    FileExtension::new(a)
  }
}

impl From<FileExtension> for String {
  fn from(file_extension: FileExtension) -> String {
    file_extension.file_extension
  }
}


#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "hashmap_json", derive(Debug))]
pub enum VarValue {
  Bool(bool),
  String(String),
  F64(f64),
  U32(u32),
  Vec(Vec<VarValue>),
  HashMap(HashMap<String, VarValue>),
}

impl VarValue {
  pub fn is_truthy(&self) -> bool {
    match self {
      Self::Bool(boolean) => *boolean,
      Self::String(string) => string == "",
      Self::F64(decimal) => *decimal != 0.0,
      Self::U32(integer) => *integer != 0,
      Self::Vec(vector) => vector.len() > 0,
      Self::HashMap(hashmap) => hashmap.keys().len() > 0,
    }
  }
}

impl fmt::Display for VarValue {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      VarValue::Bool(boolean) => write!(f, "{}", boolean.to_string()),
      VarValue::String(string) => write!(f, "{}", string),
      VarValue::F64(decimal) => write!(f, "{}", decimal.to_string()),
      VarValue::U32(integer) => write!(f, "{}", integer.to_string()),
      VarValue::Vec(vector) => write!(f, "{:?}", vector.iter().map(|a| format!("{}", a)).collect::<Vec<String>>()),
      VarValue::HashMap(_hashmap) => write!(f, "Enable the `hashmap_json` crate feature"),
    }
  }
}

#[cfg(feature = "hashmap_json")]
impl fmt::Display for VarValue {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      VarValue::Bool(boolean) => write!(f, "{}", boolean.to_string()),
      VarValue::String(string) => write!(f, "{}", string),
      VarValue::F64(decimal) => write!(f, "{}", decimal.to_string()),
      VarValue::U32(integer) => write!(f, "{}", integer.to_string()),
      VarValue::Vec(vector) => write!(f, "{:?}", vector.iter().map(|a| format!("{}", a)).collect::<Vec<String>>()),
      VarValue::HashMap(hashmap) => write!(f, "{}", serde_json::to_string(hashmap)),
    }
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SyntaxMatch {
  pub content: String,
  pub index: usize, //start index in text
}

pub struct ForLoopInfo {
  index: usize,
  total: usize,
  current: usize,
  var_value: Vec<VarValue>, //value we are looping over
  iter_var_name: Option<String>,
  index_var_name: Option<String>,
}

pub type Vars = HashMap<String, VarValue>;

pub struct Renderer {
  pub templates_dir: String,
  pub components_dir: String,
  pub file_extension: FileExtension,
}

impl Renderer {
  pub fn new(templates_dir: String, components_dir: String, file_extension: FileExtension) -> Self {
    Self {
      templates_dir,
      components_dir,
      file_extension,
    }
  }

  pub fn concat_path(path1: &String, path2: &String) -> String {
    if path1.ends_with("/") && path2.starts_with("/") {
      let mut path1: String = path1.clone();
      path1.truncate(path1.len()-1);
      format!("{}{}", path1, path2)
    } else if !path1.ends_with("/") && !path2.starts_with("/") {
      format!("{}/{}", path1, path2)
    } else {
      format!("{}{}", path1, path2)
    }
  }

  pub fn sanitize(text: &String) -> String {
    text.replace("<", "&lt;").replace(">", "&gt;")
  }

  pub fn check_var_name_legality(var_name: &String, dot_allowed: bool) -> Result<(), ErrorKind> {
    let mut legal_chars: Vec<char> = vec!['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '_', '/', '.'];
    if !dot_allowed {
      legal_chars.pop();
    }
    //if any of them are not in the legal chars
    let fail: bool = var_name.chars().any(|c| !legal_chars.contains(&c.to_ascii_lowercase()));
    if fail {
      Err(ErrorKind::IllegalVarName(var_name.clone()))
    } else {
      Ok(())
    }
  }

  pub fn find_syntax_matches(template_content: &String) -> Vec<SyntaxMatch> {
    let legal_chars: Vec<char> = vec!['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '_', '.', ':', '-', '!'];
    let mut matches: Vec<SyntaxMatch> = Vec::new();
    //"[[  ]]"
    let chars: Vec<char> = template_content.chars().collect();
    let mut in_match: bool = false;
    let mut match_index: usize = 0; //start index of match
    for index in 0..chars.len() {
      let current_char: char = chars[index];
      if index > 1 && index < chars.len()-2 {
        if current_char == ' ' && chars[index-1] == '[' && chars[index-2] == '[' {
          in_match = true;
          match_index = index-2;
        } else if in_match && chars[index] == ' ' && chars[index+1] == ']' && chars[index+2] == ']' {
          in_match = false;
          matches.push(SyntaxMatch {
            index: match_index,
            content: chars[match_index..index+3].iter().collect(),
          });
        } else if in_match && !legal_chars.contains(&current_char.to_ascii_lowercase()) {
          in_match = false;
        }
      }
    }
    matches
  }

  pub fn get_var(var_name: String, vars: &Vars) -> Result<&VarValue, ErrorKind> {
    Self::check_var_name_legality(&var_name, true)?;
    let mut parts: VecDeque<&str> = var_name.split(".").into_iter().collect();
    let part_uno: &str = parts.pop_front().unwrap();
    let var_value_unwrapped: &Option<&VarValue> = &vars.get(part_uno);
    if var_value_unwrapped.is_none() {
      //bad
      return Err(ErrorKind::VarNotFound(var_name));
    }
    let mut var_value = var_value_unwrapped.unwrap();
    for part in parts {
      if let VarValue::HashMap(var_value_hashmap) = &var_value {
        let var_value_hashmap_unwrapped: Option<&VarValue> = var_value_hashmap.get(part);
        if var_value_hashmap_unwrapped.is_none() {
          //bad
          return Err(ErrorKind::VarNotFound(var_name));
        }
        var_value = var_value_hashmap_unwrapped.unwrap();
      } else {
        //bad
        return Err(ErrorKind::VarNotFound(var_name));
      }
    }
    Ok(var_value)
  }

  pub fn render(&self, template_contents: String, vars: &mut Vars, recursion_layer: Option<usize>) -> Result<String, ErrorKind> {
    let recursion_layer: usize = recursion_layer.unwrap_or(0);
    let syntax_matches: Vec<SyntaxMatch> = Self::find_syntax_matches(&template_contents);
    if syntax_matches.len() == 0 {
      return Ok(template_contents);
    }
    let mut rendered: String = template_contents[0..syntax_matches[0].index].to_string();
    let mut for_loops: Vec<ForLoopInfo> = vec![];
    let mut index: usize = 0;
    let mut iterations_: usize = 0;
    loop {
      if index == syntax_matches.len() {
        break;
      }
      if iterations_ > 75000 {
        println!("Passed 75000 iterations while rendering, infinite loop?");
      }
      let syntax_match: &SyntaxMatch = &syntax_matches[index];
      let exp_parts: Vec<&str> = syntax_match.content[3..syntax_match.content.len()-3].split(":").collect();
      if exp_parts.len() < 1 {
        return Err(ErrorKind::BadArgument("An empty '[[ ]]' is not valid".to_string()));
      }
      if exp_parts[0] == "component" {
        //we do not want get into an infinite recursion loop with components referring to each other
        if recursion_layer > 5 {
          return Err(ErrorKind::RecursionTooDeep);
        }
        if exp_parts.len() != 2 {
          return Err(ErrorKind::BadArgument("`component:` statement missing component name (second arg), or more than two args".to_string()));
        }
        let mut file_name: String = exp_parts[1].to_string();
        if !file_name.contains(".") {
          file_name += self.file_extension.get_string_ref();
        }
        rendered += &self.render_template(Self::concat_path(&self.components_dir, &file_name), vars, Some(recursion_layer+1))?;
      } else if exp_parts[0] == "for" {
        let mut already_exists: bool = false;
        let most_recent: Option<&ForLoopInfo> = for_loops.last();
        if most_recent.is_some() {
          if most_recent.unwrap().index == index {
            //for loop already exists, just continue and do nothing
            already_exists = true;
          }
        }
        if !already_exists {
          //variables in for loops are not scoped because that would be too much work
          if exp_parts.len() < 2 {
            return Err(ErrorKind::BadArgument("`for:` statement missing variable name to loop over (second arg)".to_string()));
          }
          let var_name: String = exp_parts[1].to_string();
          let var_value: &VarValue = Self::get_var(var_name, &vars)?;
          if let VarValue::Vec(vec_value) = var_value {
            let vec_value_cloned = vec_value.clone();
            let vec_length: usize = vec_value.len().clone();
            let iter_var_name: Option<String>;
            if exp_parts.len() >= 3 {
              //set iter variable (optional) (you know, the "post" in "for post in posts")
              //(I don't know what the actual name of that thing is)
              let iter_var_name_: String = exp_parts[2].to_string();
              Self::check_var_name_legality(&iter_var_name_, false)?;
              iter_var_name = Some(iter_var_name_.clone());
              //if vec is empty, that is handled later on
              if vec_value.len() > 0 {
                vars.insert(iter_var_name_, vec_value[0].clone());
              }
            } else {
              iter_var_name = None;
            }
            let index_var_name: Option<String>;
            if exp_parts.len() >= 4 {
              //set index count
              let index_var_name_: String = exp_parts[3].to_string();
              Self::check_var_name_legality(&index_var_name_, false)?;
              index_var_name = Some(index_var_name_.clone());
              vars.insert(index_var_name_, VarValue::U32(0));
            } else {
              index_var_name = None;
            }
            if exp_parts.len() >= 5 {
              //set max count
              let max_var_name: String = exp_parts[4].to_string();
              Self::check_var_name_legality(&max_var_name, false)?;
              vars.insert(max_var_name, VarValue::U32(vec_length as u32-1));
            }
            for_loops.push(ForLoopInfo {
              index,
              total: vec_length,
              current: 0,
              var_value: vec_value_cloned,
              iter_var_name,
              index_var_name,
            });
            //make sure thing we are iterating over isn't empty
            if vec_length == 0 {
              //skip straight to the endfor
              let sliced: Vec<SyntaxMatch> = syntax_matches[index+1..syntax_matches.len()].to_vec();
              let mut new_index: Option<usize> = None;
              let mut extra_fors: usize = 0;
              for i in 0..sliced.len() {
                let match_content: &String = &sliced[i].content;
                if match_content.starts_with("[[ for:") {
                  extra_fors += 1;
                } else if match_content == "[[ endfor ]]" {
                  if extra_fors == 0 {
                    new_index = Some(i);
                    break;
                  }
                  extra_fors -= 1;
                }
              }
              if new_index.is_none() {
                //`for:` statement missing `[[ endfor ]]`
                return Err(ErrorKind::MissingEndFor);
              }
              index += new_index.unwrap()+1;
              continue;
            }
          } else {
            return Err(ErrorKind::BadArgument("variable being looped over in `for:` statement is not a vector".to_string()));
          }
        }
      } else if exp_parts[0] == "endfor" {
        //check if for loop is over, if not, go back to for
        let for_loops_len: usize = for_loops.len();
        let current_loop: &mut ForLoopInfo = &mut for_loops[for_loops_len-1];
        current_loop.current += 1;
        if current_loop.current >= current_loop.total {
          //for loop ended, onwards! oh yeah, also remove the current for loop info
          for_loops.pop();
        } else {
          //update iter var
          if current_loop.iter_var_name.is_some() {
            vars.insert(current_loop.iter_var_name.clone().unwrap(), current_loop.var_value[current_loop.current].clone());
          }
          if current_loop.index_var_name.is_some() {
            vars.insert(current_loop.index_var_name.clone().unwrap(), VarValue::U32(current_loop.current.clone() as u32));
          }
          //go back to start of for loop index
          index = current_loop.index;
          continue;
        }
      } else if exp_parts[0] == "if" {
        if exp_parts.len() < 2 {
          return Err(ErrorKind::BadArgument("`if:` statement missing variable name (second arg)".to_string()));
        }
        let var_name: String = exp_parts[1].to_string();
        let var_value: &VarValue = Self::get_var(var_name, &vars)?;
        let condition_pass: bool;
        if exp_parts.len() == 2 {
          //make sure var is truthy
          if var_value.is_truthy() {
            condition_pass = true;
          } else {
            condition_pass = false;
          }
        } else if exp_parts.len() == 3 {
          //compare with second var
          let mut var_name2: String = exp_parts[2].to_string();
          let mut if_not: bool = false;
          if var_name2.starts_with("!") {
            var_name2 = var_name2[1..var_name2.len()].to_string();
            if_not = true;
          }
          let var_value2: &VarValue = Self::get_var(var_name2, &vars)?;
          if if_not {
            //make sure the two compared variables are NOT equal
            if var_value != var_value2 {
              condition_pass = true;
            } else {
              condition_pass = false;
            }
          } else {
            //regular comparison statement
            if var_value == var_value2 {
              condition_pass = true;
            } else {
              condition_pass = false;
            }
          }
        } else {
          return Err(ErrorKind::BadArgument("`if:` statement cannot have more than 3 args".to_string()));
        }
        if !condition_pass { //failed condition
          //skip to the endif
          let sliced: Vec<SyntaxMatch> = syntax_matches[index+1..syntax_matches.len()].to_vec();
          let mut new_index: Option<usize> = None;
          let mut extra_ifs: usize = 0;
          for i in 0..sliced.len() {
            let match_content: &String = &sliced[i].content;
            if match_content.starts_with("[[ if:") {
              extra_ifs += 1;
            } else if match_content == "[[ endif ]]" {
              if extra_ifs == 0 {
                new_index = Some(i);
                break;
              }
              extra_ifs -= 1;
            }
          }
          if new_index.is_none() {
            //`if:` statement missing `[[ endif ]]`
            return Err(ErrorKind::MissingEndIf);
          }
          index += new_index.unwrap()+1;
          continue;
        }
      } else if exp_parts[0] == "endif" {
        //yup, nothing here
      } else { //html:<variable name> or <variable name>
        //variable
        let var_name: String;
        if exp_parts[0] == "html" {
          if exp_parts.len() != 2 {
            return Err(ErrorKind::BadArgument("`html:` statement missing variable name, the second arg, or has more than two args".to_string()));
          }
          var_name = exp_parts[1].to_string();
        } else {
          var_name = exp_parts[0].to_string();
        }
        //convert to string
        let var_value_string: String = (Self::get_var(var_name, &vars)?).to_string();
        //add indentation
        let current_last = rendered.split("\n").last().unwrap();
        let mut indentation: usize = 0;
        for i in 0..current_last.len() {
          if current_last.chars().nth(i).unwrap() != ' ' {
            break;
          }
          indentation += 1;
        }
        let mut var_lines: VecDeque<&str> = var_value_string.split("\n").into_iter().collect();
        let var_first: &str = var_lines.pop_front().unwrap();
        //append spaces
        let var_value: String;
        if var_lines.len() == 0 {
          var_value = var_first.to_string();
        } else {
          var_value = format!("{}\n{}", var_first, var_lines.into_iter().map(
            |var_line| {
              " ".repeat(indentation)+var_line
            }
          ).collect::<Vec<String>>().join("\n"));
        }
        if exp_parts[0] == "html" {
          //variable but not sanitized
          rendered += &var_value;
        } else {
          rendered += &Self::sanitize(&var_value);
        }
      }
      if index != syntax_matches.len()-1 {
        //add the html that comes after this, up until the next template syntax match thing
        rendered += &template_contents[syntax_match.index+syntax_match.content.len()..syntax_matches[index+1].index];
      } else {
        //last index, add all the way till end of template
        rendered += &template_contents[syntax_match.index+syntax_match.content.len()..template_contents.len()];
      }
      index += 1;
      iterations_ += 1;
    }
    Ok(rendered)
  }

  pub fn render_template(&self, template_name: String, vars: &mut Vars, recursion_layer: Option<usize>) -> Result<String, ErrorKind> {
    let mut template_file_name = template_name;
    if !template_file_name.contains(".") {
      template_file_name += self.file_extension.get_string_ref();
    }
    let content: String = fs::read_to_string(Self::concat_path(&self.templates_dir, &template_file_name)).unwrap();
    self.render(content, vars, recursion_layer)
  }
}
