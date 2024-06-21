use toml::{Table, Value};

#[derive(Debug)]
pub enum SearchType {
    ArrayIndex(usize),
    FieldKey(String),

}

#[derive(Debug)]
pub enum SearchTypeError {
    ArrayIndexError, FieldKeyError
}

pub fn get_key(path: Vec<SearchType>, toml_table: &Value) -> Result<Value, SearchTypeError> {
    let mut next_value = Some(toml_table);

    for search_type_value in path.iter() {
        match &next_value {
            Some(Value::Array(a)) => {
                if !matches!(search_type_value, SearchType::ArrayIndex(_)) {
                    return Err(SearchTypeError::ArrayIndexError);
                } else if let SearchType::ArrayIndex(i) = search_type_value {
                    match a.get(*i)
                        .map(|f| Ok(f))
                        .or(Some(Err(SearchTypeError::ArrayIndexError)))
                        .unwrap() {
                        Ok(f) => {
                            next_value = Some(f)
                        }
                        e @ Err(_) => {
                            return e.cloned()
                        }
                    }
                }
            }
            Some(Value::Table(t)) => {
                if let Some(value) = get_field_key(search_type_value, &t) {
                    match value {
                        Ok(v) => {
                            next_value = Some(v);
                        }
                        e @ Err(_) => {
                            return e.cloned()
                        }
                    }
                }
            }
            _ => {
            }
        }
    }

    next_value.cloned()
        .map(|v| Ok(v)).or(Some(Err(SearchTypeError::ArrayIndexError)))
        .unwrap()
}

fn get_field_key<'a>(p: &'a SearchType, t: &'a Table) -> Option<Result<&'a Value, SearchTypeError>> {
    if !matches!(p, SearchType::FieldKey(_)) {
        return Some(Err(SearchTypeError::FieldKeyError));
    } else if let SearchType::FieldKey(f) = p {
        if !t.contains_key(f) {
            return Some(Err(SearchTypeError::FieldKeyError));
        }
        return t.get(f.to_string().as_str()).map(|v| Ok(v))
    }
    None
}