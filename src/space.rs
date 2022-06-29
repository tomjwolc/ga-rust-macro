extern crate proc_macro;

use proc_macro::*;
use proc_macro::TokenTree::*;

use phf::phf_map;

use rand::prelude::*;

use std::f64::consts::*;

use fancy_regex::Regex;

use lazy_static::lazy_static;

static PEMDAS: phf::Map<char, usize> = phf_map! {
    '+' => 1,
    '-' => 1,
    '*' => 2,
    '|' => 2,
    '&' => 2,
    '%' => 2,
    '>' => 2,
    '/' => 2,
    // '^' => 3, // Nope
    '@' => 4,
    '~' => 5,
    '!' => 5,
};

static EQ_CONSTS: phf::Map<&'static str, fn(usize) -> String> = phf_map! {
    "RAND" => |_index| {let mut rng = rand::thread_rng(); format!("{}", rng.gen::<f64>())},
    "ONES" => |_index| String::from("1.0"),
    "e"    => | index| format!("{}", if index == 0 { E   } else { 0.0 } ),
    "pi"   => | index| format!("{}", if index == 0 { PI  } else { 0.0 } ),
    "tau"  => | index| format!("{}", if index == 0 { TAU } else { 0.0 } ),
};

static FUNCS: phf::Map<&'static str, fn(Vec<String>) -> (Vec<String>, Vec<String>)> = phf_map! {
    "motor" => |num| {
        let (id1, id2) = (get_rand_def_id(), get_rand_def_id());

        let magnitude = format!("(({}) as f64).powf(0.5)", num.iter().fold(String::from("0.0"), |acc, part| match (is_zero(&acc), is_zero(part)) { 
            (  true  ,  true  ) => String::from("0.0"),
            (  false ,  true  ) => acc,
            (  true  ,  false ) => format!("(({}) as f64).powf(2.0)", part),
            (  false ,  false ) => format!("{} + (({}) as f64).powf(2.0)", acc, part)
        }));
        let mut res: Vec<String> = num.into_iter().map(|part| match is_zero(&part) {
            true  => String::from("0.0"),
            false => format!("{} * {} / {}", id2, part, id1)
        }).collect(); 

        res[0] = format!("{} + ({}).cos()", res[0], magnitude);

        (vec![format!("let {}: f64 = {}; let {} = {}.sin();", id1, magnitude, id2, id1)], res)
    },
    "norm" => |num| {
        let id = get_rand_def_id();

        let magnitude = format!("(({}) as f64).powf(0.5)", num.iter().fold(String::from("0.0"), |acc, part| match (is_zero(&acc), is_zero(part)) { 
            (  true  ,  true  ) => String::from("0.0"),
            (  false ,  true  ) => acc,
            (  true  ,  false ) => format!("(({}) as f64).powf(2.0)", part),
            (  false ,  false ) => format!("{} + (({}) as f64).powf(2.0)", acc, part)
        }));
        let mut res: Vec<String> = num.into_iter().map(|part| match is_zero(&part) {
            true  => String::from("0.0"),
            false => format!("{} / {}", part, id)
        }).collect(); 

        (vec![format!("let {}: f64 = {};", id, magnitude)], res)
    },
    "norm_w" => |num| {
        let id = get_rand_def_id();

        let magnitude = format!("(({}) as f64).powf(0.5)", FUNCS.get("weight").unwrap()(num.iter().map(|x|x.clone()).collect()).1.iter().fold(String::from("0.0"), |acc, part| match (is_zero(&acc), is_zero(part)) { 
            (  true  ,  true  ) => String::from("0.0"),
            (  false ,  true  ) => acc,
            (  true  ,  false ) => format!("(({}) as f64).powf(2.0)", part),
            (  false ,  false ) => format!("{} + (({}) as f64).powf(2.0)", acc, part)
        }));

        let mut res: Vec<String> = num.into_iter().map(|part| match is_zero(&part) {
            true  => String::from("0.0"),
            false => format!("{} / {}", part, id)
        }).collect(); 

        (vec![format!("let {}: f64 = {};", id, magnitude)], res)
    },
    "norm_b" => |num| {
        let id = get_rand_def_id();

        let magnitude = format!("(({}) as f64).powf(0.5)", FUNCS.get("bulk").unwrap()(num.iter().map(|x|x.clone()).collect()).1.iter().fold(String::from("0.0"), |acc, part| match (is_zero(&acc), is_zero(part)) { 
            (  true  ,  true  ) => String::from("0.0"),
            (  false ,  true  ) => acc,
            (  true  ,  false ) => format!("(({}) as f64).powf(2.0)", part),
            (  false ,  false ) => format!("{} + (({}) as f64).powf(2.0)", acc, part)
        }));

        let mut res: Vec<String> = num.into_iter().map(|part| match is_zero(&part) {
            true  => String::from("0.0"),
            false => format!("{} / {}", part, id)
        }).collect(); 

        (vec![format!("let {}: f64 = {};", id, magnitude)], res)
    },
    "bulk" => |num| {
        let dims = (num.len() as f64).log2() as usize;
        let grade_distr = pascal_triangle(dims + 1);
        let mut res = vec![String::from("0.0"); num.len()];
        let mut threshold = grade_distr[0];
        let mut grade = 0;

        for i in 0..num.len() {
            if i >= threshold { grade += 1; threshold += grade_distr[grade]; }

            if grade_distr[grade] * grade / (dims + 1) < grade_distr[grade] + i - threshold || grade == 0 {
                res[i] = num[i].clone();
            }
        };

        (Vec::new(), res)
    },
    "weight" => |num| {
        let dims = (num.len() as f64).log2() as usize;
        let grade_distr = pascal_triangle(dims + 1);
        let mut res = vec![String::from("0.0"); num.len()];
        let mut threshold = grade_distr[0];
        let mut grade = 0;

        for i in 0..num.len() {
            if i >= threshold { grade += 1; threshold += grade_distr[grade]; }

            if grade_distr[grade] * grade / (dims + 1) >= grade_distr[grade] + i - threshold && grade != 0 {
                res[i] = num[i].clone();
            }
        };

        (Vec::new(), res)
    }
};

// 5 letters remove: e, a, n
// 6 letters remove: e, t
static DEF_ID_CHARS: &'static str = "bcdfghijklmopqrsuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_"; // {e, a, n, t} gone

pub fn eq_macro_logic_peek(space: (usize, usize, usize), tokens: TokenStream) -> TokenStream {
    let mut str = eq_macro_logic(space, tokens).to_string();

    let bound_signifiers = if str.chars().nth(0) == Some('[') || str.chars().nth(0) == Some('{') {"---"} else {""};

    str = format!("\"{}{}{}\"", bound_signifiers, str, bound_signifiers).replace("\n", "");

    /*
    let mut offset: i64 = 0;
    let mut tabs = 0;

    for mat in MACRO_FMT_LOCS.find_iter(str.clone().as_str()).map(|mat| mat.expect("find_iter weirdness (peek)")) {
        let start = (offset + mat.start() as i64) as usize;
        let end = (offset + mat.end() as i64) as usize;

        str = match &str[start..end] {
            "---[" => { tabs += 4; offset += (tabs as i64) - 2; format!("{}[\n{}{}"  , &str[..start], " ".repeat(tabs), &str[end..])},
            "---{" => { tabs += 4; offset += (tabs as i64) - 2; format!("{}{{\n{}{}" , &str[..start], " ".repeat(tabs), &str[end..])},
            "]---" => { tabs -= 4; offset += (tabs as i64) - 2; format!("{}\n]{}{}"  , &str[..start], " ".repeat(tabs), &str[end..])},
            "}---" => { tabs -= 4; offset += (tabs as i64) - 2; format!("{}\n}}{}{}" , &str[..start], " ".repeat(tabs), &str[end..])},
            ","    => {            offset += tabs as i64;       format!("{},\n{}{}"  , &str[..start], " ".repeat(tabs), &str[end..])},
            _      => str
        };
    }
    */

    str
        .replace("---[", "[,")
        .replace("]---", "\n]")
        .replace("---{", "{,")
        .replace("}---", "\n}")
        .replace(" ", "")
        .replace(",", ",\n    ")
        .replace("asf64", " as f64")
        .replace("asisize", " as isize")
        .as_str()
        .parse()
        .expect("eq_peek: parsing failed")
}

pub fn eq_macro_logic(space: (usize, usize, usize), mut tokens: TokenStream) -> TokenStream {
    lazy_static! {
        static ref IMPL_MULT_REGEX: Regex = Regex::new(r"(?<=[0-9])(?=[a-zA-Z])").unwrap();
        static ref SIGNED_COEF_REGEX: Regex = Regex::new(r"[^0-9a-zA-Z ][ ]*[+-][ ]*[0-9]+").unwrap();
    }

    let mut token_str = tokens.to_string();

    token_str = IMPL_MULT_REGEX.replace_all(token_str.as_str(), "*").to_string();

    let mut offset = 0;

    for mat in SIGNED_COEF_REGEX.find_iter(token_str.clone().as_str()).map(|mat| mat.expect("find_iter weirdness")) {
        let start = offset + mat.start() + 1;
        let end = offset + mat.end();

        token_str = format!("{}({}){}", &token_str[..start], &token_str[start..end], &token_str[end..]);

        offset += 2;
    }

    tokens = token_str.parse().expect("Could not parse tokens after implied multiplication and signed coeficients regexes");

    // return format!("\"{}\"", tokens.to_string()).as_str().parse().unwrap();

    let (
        cayley, 
        labels
    ) = get_cayley(space);

    let mut token_vec: Vec<TokenTree> = tokens.into_iter().collect();
    let mut num_formatter: fn(Vec<String>) -> String = |num| {
        format!("[{}]",
            num[1..]
                .iter()
                .fold(format!("{}", num[0]), |acc, part| format!("{}, {}", acc, part))
        )   
    };

    if let Some(Punct(ref punct)) = token_vec.get(1) {
        if punct.as_char() == ':' {
            match token_vec[0].to_string().as_str() {
                "int" => num_formatter = |num| format!("({} as isize)", num[0]),
                "float" => num_formatter = |num| format!("{}", num[0]),
                _ => ()
            }

            token_vec = token_vec[2..].to_vec();
        }
    }

    tokens = token_vec.into_iter().collect();

    let (defs, result) = simplify(&tokens, &cayley, &labels);

    format!("{{{}{}}}", defs.iter().fold(String::new(), |acc, def| format!("{}{}", acc, def)), num_formatter(result))
        .as_str()
        .parse()
        .expect("final eq macro parse failed")
}

fn simplify(tokens: &TokenStream, cayley: &Vec<Vec<(usize, f64, f64, f64)>>, labels: &Vec<String>) -> (Vec<String>, Vec<String>) {
    let mut waiting_ops: Vec<char> = Vec::new();
    let mut waiting_nums: Vec<Vec<String>> = Vec::new();
    let mut last_pemdas: usize = 0;
    let mut function: Option<&fn(Vec<String>) -> (Vec<String>, Vec<String>)> = None;
    let mut definitions: Vec<String> = Vec::new();

    for ref token in tokens.clone().into_iter() {
        match token {
            Punct(punct) => {
                let mut pemdas = *PEMDAS.get(&punct.as_char()).expect(format!("char '{}' is not recognized", punct.as_char()).as_str());

                if waiting_ops.len() + 1 != waiting_nums.len() {
                    waiting_nums.push(vec![String::from("0.0"); cayley.len()]);
                    pemdas = 5;
                }

                if pemdas <= last_pemdas {
                    waiting_nums = vec![wrap_parens(parse_ops(waiting_ops, waiting_nums, cayley))];
                    waiting_ops = Vec::new();
                }

                last_pemdas = pemdas;

                waiting_ops.push(punct.as_char());
            },
            Ident(ident) => {
                // the variable is a number, not an array
                if ident.to_string().chars().nth(0).unwrap() == '_' {
                    let mut num = vec![String::from("0.0"); cayley.len()];
                    num[0] = format!("({} as f64)", &ident.to_string()[1..]);
                    waiting_nums.push(num);
                    continue;
                }

                // the ident matches a basis k-vector label
                if let Some(index) = labels.iter().position(|label| label == &ident.to_string()) {
                    let mut num = vec![String::from("0.0"); cayley.len()];
                    num[index] = String::from("1.0");
                    waiting_nums.push(num);
                    continue;
                }

                // the ident is a constant
                if let Some(func) = EQ_CONSTS.get(ident.to_string().as_str()) {
                    let mut num = vec![String::from("0.0"); cayley.len()];
                    num = num.iter().enumerate().map(|(i, _)| func(i)).collect();
                    waiting_nums.push(num);
                    continue;
                }

                // the ident is a function
                if let Some(func) = FUNCS.get(ident.to_string().as_str()) {
                    function = Some(func);
                    continue;
                }

                let symb = ident.to_string();
                let num: Vec<String> = vec![0.0; cayley.len()].iter().enumerate().map(|(i, _)| format!("{}[{}]", symb, i)).collect();

                waiting_nums.push(num);
            },
            Literal(literal) => {
                let mut num = vec![String::from("0.0"); cayley.len()];
                num[0] = literal.to_string();

                if !num[0].contains('.') {num[0] += ".0"}

                waiting_nums.push(num);
            },
            Group(group) => {
                let (mut defs, mut num) = simplify(&group.stream(), cayley, labels);

                definitions.append(&mut defs);

                if group.delimiter() == Delimiter::Bracket {
                    let magnitude = format!("(({}) as f64).powf(0.5)", num.iter().fold(String::from("0.0"), |acc, part| format!("{} + (({}) as f64).powf(2.0)", acc, part)));
                    num = vec![String::from("0.0"); cayley.len()];
                    num[0] = magnitude;
                }

                if let Some(&func) = function {
                    (defs, num) = func(num);

                    definitions.append(&mut defs)
                }

                waiting_nums.push(num);
            }
        };
    }

    if waiting_ops.len() > 0 {
        waiting_nums = vec![parse_ops(waiting_ops, waiting_nums, cayley)];
    }

    (definitions, waiting_nums.remove(0))
}

fn parse_ops(mut ops: Vec<char>, mut nums: Vec<Vec<String>>, cayley: &Vec<Vec<(usize, f64, f64, f64)>>) -> Vec<String> {
    let mut num = nums.remove(nums.len() - 2); // nums.len() >= 2
    ops = ops.into_iter().rev().collect();

    for i in 0..ops.len() {
        let num2 = nums.remove(nums.len() - 1); // nums.len() > ops.len()

        num = match ops[i] {
            '!' => num2.into_iter().rev().collect(),
            '~' => num2.iter().enumerate().map(|(i, part)| {
                let gr = get_grade(i, cayley.len()) as isize;

                format!("{}.0*({})", -2 * ((gr * (gr - 1) / 2) % 2) + 1, part)
            }).collect(),
            '-' | '+' => num.iter().enumerate().map(|(j, _)| {
                match (is_zero(&num[j]), is_zero(&num2[j])) {
                    ( true  , true  ) => String::from("0.0"),
                    ( false , true  ) => num[j].clone(),
                    ( true  , false ) => if ops[i] == '+' { num2[j].clone() } else { 
                        match num2[i].parse::<f64>() {
                            Ok(float) => to_float_string(-1.0 * float),
                            Err(_) => format!("-1.0 * {}", num2[j]) 
                        }
                    },
                    ( false , false ) => {
                        match (num[i].parse::<f64>(), num[i].parse::<f64>()) {
                            (Ok(f1), Ok(f2)) => to_float_string(if ops[i] == '+' { f1 + f2 } else { f1 - f2 }),
                            (_, _) => format!("{}{}{}", num[i], ops[i], num2[i])
                        }
                    }
                }
            }).collect(),
            '*' | '|' | '&' => {
                let mut res = vec![String::from("0.0"); cayley.len()];

                mult(cayley, ops[i], &num, &num2, &mut res);

                res
            },
            '%' => {
                let mut res = vec![String::from("0.0"); cayley.len()];

                mult(cayley, '&', &num.into_iter().rev().collect(), &num2.into_iter().rev().collect(), &mut res);

                res.into_iter().rev().collect()
            },
            '/' => {
                if !num2[1..].iter().all(|part| is_zero(part)) { 
                    panic!("Divisor: {:?} is not a real number", num2);
                }

                let mut res = vec![String::from("0.0"); cayley.len()];
                let mut multi = vec![String::from("0.0"); cayley.len()];

                multi[0] = 
                    if let Ok(float) = num2[0].parse::<f64>() {
                        (1.0 / float).to_string()
                    } else {
                        format!("(1.0 / ({}))", num2[0])
                    };


                mult(cayley, ops[i], &num, &multi, &mut res);

                res
                
            },
            '@' => {
                get_grade_slice(num, num2[0].parse::<f64>().expect("Expected an explicit integer as the second parameter for @") as usize, cayley.len())
            },
            _ => num
        };

        num = match ops.get(i + 1) {
            Some(next_op) if PEMDAS.get(next_op).unwrap() < PEMDAS.get(&ops[i]).unwrap() => num,
            Some(&next_op) if next_op == ops[i] && (next_op == '+' || next_op == '*') => num,
            None => num,
            _ => wrap_parens(num)
        };
    }

    num
}

fn mult(cayley: &Vec<Vec<(usize, f64, f64, f64)>>, op: char, v1: &Vec<String>, v2: &Vec<String>, res: &mut Vec<String>) { 
    for i in 0..cayley.len() {
        if is_zero(&v1[i]) {continue;}

        for j in 0..cayley.len() {
            let info = cayley[j][i];
            let mut coef = (match op { '*' => info.1, '|' => info.2, '&' => info.3, _ => info.1 }).to_string();

            if !coef.contains('.') { coef += ".0" }

            if is_zero(&coef) || is_zero(&v2[j]) { continue; }

            let (mut n1, mut n2) = (v1[i].clone(), v2[j].clone());

            (coef, n1, n2) = match (coef.parse::<f64>(), n1.parse::<f64>(), n2.parse::<f64>()) {
                (  Ok(f1)  ,  Ok(f2)  ,  Ok(f3)  ) => (  to_float_string(f1 * f2 * f3)  ,  String::from("1.0")              ,  String::from("1.0")  ),
                (  Ok(f1)  ,  Ok(f2)  ,  Err(_)       ) => (  to_float_string(f1 * f2)       ,  String::from("1.0")              ,  n2                   ),
                (  Ok(f1)  ,  Err(_)       ,  Ok(f3)  ) => (  to_float_string(f1 * f3)       ,  n1                               ,  String::from("1.0")  ),
                (  Err(_)       ,  Ok(f2)  ,  Ok(f3)  ) => (  coef                                  ,  to_float_string(f2 * f3)  ,  String::from("1.0")  ),
                (  _            ,  _            ,  _            ) => (  coef                                  ,  n1                               ,  n2                   )
            };

            let part = match (coef.as_str(), n1.as_str(), n2.as_str()) {
                ( "0.0" , _, _) | (_, "0.0" , _) | (_, _, "0.0" ) => continue,
                ( "-0.0", _, _) | (_, "-0.0", _) | (_, _, "-0.0") => continue,
                
                (  "1.0"   ,  "1.0"   ,  "1.0"   ) => format!("{}"             , "1.0"   ),
                (  a ,  "1.0"   ,  "1.0"   ) => format!("{}"             , a       ),
                (  "1.0"   ,  b ,  "1.0"   ) => format!("{}"             , b       ),
                (  "1.0"   ,  "1.0"   ,  c ) => format!("{}"             , c       ),
                (  a ,  b ,  "1.0"   ) => format!("{} * {}"      , a, b    ),
                (  a ,  "1.0"   ,  c ) => format!("{} * {}"      , a, c    ),
                (  "1.0"   ,  b ,  c ) => format!("{} * {}"      , b, c    ),
                (  a ,  b ,  c ) => format!("{} * {} * {}" , a, b, c )
            };

            res[info.0] = match (res[info.0].parse::<f64>(), part.parse::<f64>()) {
                (  Ok(f1)  ,  Ok(f2)  )              => to_float_string(f1 + f2),
                (  Ok(f1)   ,  _           ) if f1 == 0.0 => format!("{}", part),
                (  _            ,  Ok(f2)  ) if f2 == 0.0 => format!("{}", res[info.0]),
                (  _            ,  _            )              => format!("{} + {}", res[info.0], part)
            }
        }
    };  
}

fn is_zero(num: &String) -> bool {
    num.as_str() == "0.0" || num.as_str() == "-0.0"
}

fn get_grade(index: usize, len: usize) -> usize {
    if index == 0 {return 0}

    let grade_distr = pascal_triangle(1 + (len as f64).log2() as usize);
    let mut acc = 0;
    let mut i = 0;

    while acc < index {
        i += 1;
        
        acc += grade_distr[i];
    }

    i
}

fn get_grade_slice(num: Vec<String>, grade: usize, len: usize) -> Vec<String> {
    let grade_distr = pascal_triangle(1 + (len as f64).log2() as usize);
    let start = grade_distr[0..grade].iter().fold(0, |acc, x| acc + x);
    let end = grade_distr[0..(grade + 1)].iter().fold(0, |acc, x| acc + x);
    let mut res = vec![String::from("0.0"); len];

    for (i, n) in res.iter_mut().enumerate() {
        if start <= i && i < end { *n = num[i].as_str().to_string() }
    };

    res
}

fn pascal_triangle(depth: usize) -> Vec<usize> {
    match depth {
        0 => Vec::new(),
        1 => vec![1],
        2 => vec![1, 1],
        _ => {
            let prev_row = pascal_triangle(depth - 1);
            let mut row = vec![1];

            for i in 0..(prev_row.len() - 1) {
                row.push(prev_row[i] + prev_row[i + 1]);
            }

            row.push(1);

            row
        }
    }
}

fn to_float_string(float: f64) -> String {
    let mut str = float.to_string();
    if !str.contains('.') { str += ".0" }

    str
}

fn wrap_parens(num: Vec<String>) -> Vec<String> {
    num.into_iter().map(|part| if let Ok(_) = part.parse::<f64>() { part } else if is_wrapped(&part[..]) { part } else { format!("({})", part) }).collect()
}

fn is_wrapped(mut str: &str) -> bool {
    let mut paren_depth = 1;

    while str.len() > 1 {
        str = &str[1..];

        paren_depth += match &str[0..1] {
            "(" => 1,
            ")" => -1,
            _ => 0
        };
    }

    paren_depth == 0
}

fn get_rand_def_id() -> String {
    let mut word = String::new();

    for _ in 0..6 {
        let mut rng = rand::thread_rng();
        let r = (rng.gen::<f64>() * DEF_ID_CHARS.len() as f64) as usize;

        word = format!("{}{}", word, &DEF_ID_CHARS[r..(r+1)]);
    }

    word
}

fn get_cayley(space: (usize, usize, usize)) -> (Vec<Vec<(usize, f64, f64, f64)>>, Vec<String>) {
    let (p, n, z) = space;

    let indeces: Vec<usize> = vec![0; p + n + z].iter().enumerate().map(|en| en.0).collect();
    let mut perms: Vec<Vec<usize>> = permutations(&indeces);

    perms = sort(
        &mut perms, 
        |a, b| {
            if a.len() == b.len() {
                for i in 0..a.len() {
                    if a[i] != b[i] {return a[i] as isize - b[i] as isize}
                };

                0
            } else {
                a.len() as isize - b.len() as isize
            }
        }
    );
    
    if (p, n, z) == (2, 0, 1) { // I have to do this because pga is weird
        perms[5] = vec![2, 0];
    } else if (p, n, z) == (3, 0, 1) {
        perms[9] = vec![3, 1];
        perms[11] = vec![0, 2, 1];
        perms[13] = vec![0, 3, 2];
    }

    let mut cayley: Vec<Vec<(usize, f64, f64, f64)>> = Vec::new();

    for i in 0..perms.len() {
        cayley.push(Vec::new());

        for j in 0..perms.len() {
            let mut product: Vec<&usize> = perms[j].iter().chain(perms[i].iter()).collect();
            let mut coef: f64 = 1.0;

            let mut k1 = 0;
            let mut k2: usize;

            while k1 + 1 < product.len() { // Removes duplicates and updates coef accordingly

                k2 = k1 + 1;

                while k2 < product.len() {
                    if product[k1] == product[k2] {
                        coef *= 2.0 * ((k2 - k1) % 2) as f64 - 1.0;
                        coef *= if *product[k1] < z { 0.0 } else if *product[k1] < z + n { -1.0 } else { 1.0 };

                        product.remove(k2);
                        product.remove(k1);

                        (k1, k2) = (0, 1);
                    } else {
                        k2 += 1;
                    }
                }

                if (k1, k2) != (0, 1) {k1 += 1};
            }

            let index = perms.iter().position(|base| {
                product.iter().all(|&&n1| base.iter().position(|&n2| n1 == n2) != None)
            }).unwrap();

            for (k, n) in perms[index].iter().enumerate() {
                if product[k] != n {
                    let k_real = product.iter().position(|&x| x == n).unwrap();

                    coef *= -1.0;

                    (product[k], product[k_real]) = (product[k_real], product[k]);
                }
            }

            let yes_inner_product = 
                perms[i].iter().all(|n1| perms[j].iter().position(|n2| n1 == n2) != None) ||
                perms[j].iter().all(|n1| perms[i].iter().position(|n2| n1 == n2) != None);

            cayley[i].push((
                index, 
                coef,
                if                 yes_inner_product               { coef } else { 0.0  },
                if product.len() < perms[i].len() + perms[j].len() { 0.0  } else { coef },
            ));
        };
    };

    let labels = perms.iter().map(|base| {
        if base.len() == 0 {
            String::from("")
        } else { 
            base.iter().fold(String::from("e"), |acc, n| format!("{}{}", acc, *n))
        }
    }).collect();

    (
        cayley, 
        labels
    )
}

fn permutations<T>(arr: &Vec<T>) -> Vec<Vec<T>> 
where
    T: Copy
{
    let mut perms = vec![vec![]];
    perms.extend_from_slice(&permutations_rec_loop(arr, 0));

    perms
}

fn permutations_rec_loop<T>(arr: &Vec<T>, index: usize) -> Vec<Vec<T>> 
where
    T: Copy
{
    if index >= arr.len() {return Vec::new()}
    if index == arr.len() - 1 {return vec![vec![arr[index]]]};

    let mut res: Vec<Vec<T>> = Vec::new();

    for (i, t) in arr[index..].iter().enumerate() {
        res.push(vec![ *t ]);

        res.append(
            &mut permutations_rec_loop(arr, index + i + 1)
                .iter()
                .map(|perm| {
                    let mut new_perm: Vec<T> = vec![*t];
                    for t2 in perm { new_perm.push(*t2) };

                    new_perm
                })
                .collect()
        );
    }

    res
}

fn sort<T>(arr: &mut Vec<T>, func: fn(&T, &T) -> isize) -> Vec<T> {
    let mut indeces: Vec<usize> = vec![0; arr.len()].iter().enumerate().map(|en| en.0).collect();

    indeces = sort_with_indeces(&arr.iter().map(|x| x).collect::<Vec<&T>>()[..], &indeces[..], func);

    let mut result: Vec<T> = Vec::new();

    for i in 0..indeces.len() {
        let index = indeces[i];

        result.push(arr.remove(index));

        indeces = indeces.iter().map(|&x| if x > index {x - 1} else {x}).collect();
    }

    result
}

fn sort_with_indeces<'a, T: 'a>(arr: &[&T], indeces: &[usize], func: fn(&T, &T) -> isize) -> Vec<usize> {
    let mid = indeces.len() / 2;

    let mut arr1: Vec<usize> = if mid == 1 { vec![indeces[0]] } else { sort_with_indeces(arr, &indeces[..mid], func) };
    let mut arr2: Vec<usize> = if indeces.len() == 2 { vec![indeces[1]] } else { sort_with_indeces(arr, &indeces[mid..], func) };
    let mut result: Vec<usize> = Vec::new();

    while arr1.len() + arr2.len() > 0 {
        if arr1.len() == 0 {
            result.push(arr2.remove(0));
            continue;
        } else if arr2.len() == 0 {
            result.push(arr1.remove(0));
            continue;
        }

        if func(arr[arr1[0]], arr[arr2[0]]) < 0 {
            result.push(arr1.remove(0));
        } else {
            result.push(arr2.remove(0));
        }
    };

    result
}