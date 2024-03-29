#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum NamingCase {
    /// All chars are Lower - mycase
    Lower,
    /// All chars are Upper - MYCASE
    Upper,
    /// Begins from Upper and next Lower chars - MyCase
    Pascal,
    /// First word begins from Lower and next words using Pascal case - myCase
    Camel,
}

#[derive(Debug)]
pub enum NamingError {
    /// Name's prefix doesn't match naming's prefix
    InvalidPrefix(String),
    /// Name's suffix doesn't match naming's suffix
    InvalidSuffix(String),
    /// Name doesn't follow the naming's case
    InvalidCase(NamingCase)
}

#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Naming {
    prefix: String,
    suffix: String,
    sep: String,
    case: NamingCase
}

impl Naming {
    pub fn new(prefix: String, suffix: String, sep: String, case: NamingCase) -> Self {
        Self {
            prefix,
            suffix,
            sep,
            case
        }
    }

    /// PascalCase
    pub fn pascal_case() -> Self {
        Self {
            prefix: String::new(),
            suffix: String::new(),
            sep: String::new(),
            case: NamingCase::Pascal
        }
    }

    /// snake_case
    pub fn snake_case() -> Self {
        Self {
            prefix: String::new(),
            suffix: String::new(),
            sep: "_".to_string(),
            case: NamingCase::Lower
        }
    }

    /// CONST_CASE
    pub fn const_case() -> Self {
        Self {
            prefix: String::new(),
            suffix: String::new(),
            sep: "_".to_string(),
            case: NamingCase::Upper
        }
    }

    /// camelCase
    pub fn camel_case() -> Self {
        Self {
            prefix: String::new(),
            suffix: String::new(),
            sep: String::new(),
            case: NamingCase::Camel
        }
    }

    /// m_hungarianMember
    pub fn hungarian_member() -> Self {
        Self {
            prefix: "m_".to_string(),
            suffix: String::new(),
            sep: String::new(),
            case: NamingCase::Camel
        }
    }

    /// pHungarianParameter
    pub fn hungarian_parameter() -> Self {
        Self {
            prefix: "p".to_string(),
            suffix: String::new(),
            sep: String::new(),
            case: NamingCase::Pascal
        }
    }

    pub fn to_parts(&self, name: &str) -> Result<Vec<String>, NamingError> {
        let stripped_name = name.strip_prefix(&self.prefix)
            .ok_or(NamingError::InvalidPrefix(self.prefix.clone()))?
            .strip_suffix(&self.suffix)
            .ok_or(NamingError::InvalidSuffix(self.suffix.clone()))?;
        let mut name_parts = Vec::new();

        match self.case {
            NamingCase::Lower => {
                for part in stripped_name.split(&self.sep) {
                    if part.chars().any(|c| c.is_uppercase()) {
                        return Err(NamingError::InvalidCase(NamingCase::Lower));
                    }
                    name_parts.push(part.to_string());
                }
            }
            NamingCase::Upper => {
                for part in stripped_name.split(&self.sep) {
                    if part.chars().any(|c| c.is_lowercase()) {
                        return Err(NamingError::InvalidCase(NamingCase::Upper));
                    }
                    name_parts.push(part.to_string());
                }
            }
            NamingCase::Pascal => {
                let mut current_word = String::new();

                for (i, c) in stripped_name.chars().enumerate() {
                    if i == 0 && c.is_lowercase() {
                        return Err(NamingError::InvalidCase(NamingCase::Pascal));
                    }

                    if c.is_uppercase() {
                        if !current_word.is_empty() {
                            name_parts.push(current_word.clone());
                            current_word.clear();
                        }
                        current_word.push(c);
                    } else {
                        current_word.push(c);
                    }
                }

                if !current_word.is_empty() {
                    name_parts.push(current_word);
                }
            }
            NamingCase::Camel => {
                let mut current_word = String::new();

                for (i, c) in stripped_name.chars().enumerate() {
                    if i == 0 && c.is_uppercase() {
                        return Err(NamingError::InvalidCase(NamingCase::Camel));
                    }

                    if c.is_uppercase() {
                        name_parts.push(current_word.clone());
                        current_word.clear();
                        current_word.push(c);
                    } else {
                        current_word.push(c);
                    }
                }

                if !current_word.is_empty() {
                    name_parts.push(current_word);
                }
            }
        }

        Ok(name_parts)
    }

    pub fn from_parts(&self, parts: &[String]) -> String {
        let result = match self.case {
            NamingCase::Lower => parts.iter().map(|part| part.to_lowercase()).collect::<Vec<String>>().join(&self.sep),
            NamingCase::Upper => parts.join(&self.sep).to_uppercase(),
            NamingCase::Pascal => {
                let mut name = String::new();
                for part in parts {
                    let mut chars = part.chars();
                    if let Some(first_char) = chars.next() {
                        name.push(first_char.to_uppercase().next().unwrap());
                        name.push_str(chars.as_str().to_lowercase().as_str());
                    }
                }
                name
            },
            NamingCase::Camel => {
                let mut name = String::new();
                for (i, part) in parts.iter().enumerate() {
                    if i == 0 {
                        name.push_str(&part.to_lowercase());
                    }
                    else {
                        let mut chars = part.chars();
                        if let Some(first_char) = chars.next() {
                            name.push(first_char.to_uppercase().next().unwrap());
                            name.push_str(chars.as_str().to_lowercase().as_str());
                        }
                    }
                }
                name
            }
        };
        [self.prefix.clone(), result, self.suffix.clone()].concat()
    }

    /// Convert given name from given naming to `self` naming and return resulted string.
    /// If name doesn't follow `naming`, it returns [NamingError].
    ///
    /// ### Example
    /// ```rs
    /// let camel_name = "myClass";
    /// let pascal_name = PASCAL_CASE.from(camel_name, &CAMEL_CASE).unwrap();
    /// assert_eq!(pascal_name, "MyClass");
    /// ```
    pub fn from(&self, name: &str, naming: &Naming) -> Result<String, NamingError> {
        let parts_result = naming.to_parts(name)?;
        Ok(self.from_parts(parts_result.as_slice()))
    }
}

#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct NamingConventions {
    pub package: Naming,
    /// String that's using to connect parts of package naming
    /// Like '-' in 'tangara-highlevel'
    pub package_divider: String,
    pub namespace: Naming,
    /// String that's using to connect parts of namespace
    /// Like '::' in Rust/C++ or '.' in C#/Java
    pub namespace_divider: String,
    pub base_type: Naming,
    pub interface: Naming,
    pub private_field: Naming,
    pub private_static: Naming,
    pub method: Naming,
    pub property: Naming,
    /// Enum variant's naming convention
    pub variant: Naming,
    /// Naming of method's or constructor's parameter
    pub parameter: Naming,
}

impl NamingConventions {
    pub fn rust() -> Self {
        Self {
            package: Naming::snake_case(),
            package_divider: "-".to_string(),
            namespace: Naming::snake_case(),
            namespace_divider: "::".to_string(),
            base_type: Naming::pascal_case(),
            interface: Naming::pascal_case(),
            private_field: Naming::snake_case(),
            private_static: Naming::const_case(),
            method: Naming::snake_case(),
            property: Naming::snake_case(),
            variant: Naming::pascal_case(),
            parameter: Naming::snake_case(),
        }
    }

    pub fn csharp() -> Self {
        let mut interface = Naming::pascal_case();
        interface.prefix = "I".to_string();
        let mut private_member = Naming::camel_case();
        private_member.prefix = "_".to_string();
        let mut private_static = Naming::camel_case();
        private_static.prefix = "s_".to_string();
        Self {
            package: Naming::pascal_case(),
            package_divider: ".".to_string(),
            namespace: Naming::pascal_case(),
            namespace_divider: ".".to_string(),
            base_type: Naming::pascal_case(),
            interface,
            private_field: private_member,
            private_static,
            method: Naming::pascal_case(),
            property: Naming::pascal_case(),
            variant: Naming::pascal_case(),
            parameter: Naming::camel_case(),
        }
    }

    /// Convert type with its namespace from given to `self` naming
    pub fn convert_type(&self, name: &str, naming: &NamingConventions) -> Result<String, NamingError> {
        let mut name_parts = name.split(&naming.namespace_divider)
            .map(|part| part.to_string()) // map from soyjak '&str' to chad 'String'
            .collect::<Vec<String>>();
        for i in 0..name_parts.len() - 1 {
            name_parts[i] = self.namespace.from(&name_parts[i], &naming.namespace)?;
        }
        let type_name = name_parts.last_mut().expect("At least one part of name must exists");
        *type_name = self.base_type.from(type_name, &naming.base_type)?; // FIXME: maybe it's interface
        Ok(name_parts.join(&self.namespace_divider))
    }

    pub fn convert_package(&self, name: &str, naming: &NamingConventions) -> Result<String, NamingError> {
        let mut parts = vec![];
        for part in name.split(&naming.package_divider) {
            let converted = self.package.from(part, &naming.package)?;
            parts.push(converted);
        }
        Ok(parts.join(&self.package_divider))
    }
}