#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;
    use tangara_highlevel::{Naming, NamingConventions};

    lazy_static! {
        static ref SNAKE_CASE: Naming = Naming::snake_case();
        static ref PASCAL_CASE: Naming = Naming::pascal_case();
        static ref HUNGARIAN_MEMBER: Naming = Naming::hungarian_member();
        static ref HUNGARIAN_PARAMETER: Naming = Naming::hungarian_parameter();
        static ref CONST_CASE: Naming = Naming::const_case();
        static ref CAMEL_CASE: Naming = Naming::camel_case();

        static ref RUST_NAMING: NamingConventions = NamingConventions::rust();
        static ref CSHARP_NAMING: NamingConventions = NamingConventions::csharp();
    }

    #[test]
    fn test_to_parts() {
        let name_parts = SNAKE_CASE.to_parts("my_field").unwrap();
        assert_eq!(name_parts[0], "my");
        assert_eq!(name_parts[1], "field");

        let name_parts = PASCAL_CASE.to_parts("MyClass").unwrap();
        assert_eq!(name_parts[0], "My");
        assert_eq!(name_parts[1], "Class");

        let name_parts = CONST_CASE.to_parts("MY_CONST").unwrap();
        assert_eq!(name_parts[0], "MY");
        assert_eq!(name_parts[1], "CONST");

        let name_parts = HUNGARIAN_MEMBER.to_parts("m_someMember").unwrap();
        assert_eq!(name_parts[0], "some");
        assert_eq!(name_parts[1], "Member");

        let name_parts = HUNGARIAN_PARAMETER.to_parts("pSomeParam").unwrap();
        assert_eq!(name_parts[0], "Some");
        assert_eq!(name_parts[1], "Param");
    }

    #[test]
    fn test_from_parts() {
        let name = SNAKE_CASE.from_parts(&["My".to_string(), "Field".to_string()]);
        assert_eq!(name, "my_field");

        let name = PASCAL_CASE.from_parts(&["my".to_string(), "class".to_string()]);
        assert_eq!(name, "MyClass");

        let name = CONST_CASE.from_parts(&["my".to_string(), "const".to_string()]);
        assert_eq!(name, "MY_CONST");

        let name = HUNGARIAN_MEMBER.from_parts(&["some".to_string(), "member".to_string()]);
        assert_eq!(name, "m_someMember");

        let name = HUNGARIAN_PARAMETER.from_parts(&["some".to_string(), "param".to_string()]);
        assert_eq!(name, "pSomeParam");

        let name = CAMEL_CASE.from_parts(&["SOME".to_string(), "MEMBER".to_string()]);
        assert_eq!(name, "someMember");
    }

    #[test]
    fn test_between_namings() {
        let camel_name = "myClass";
        let pascal_name = PASCAL_CASE.from(camel_name, &CAMEL_CASE).unwrap();
        assert_eq!(pascal_name, "MyClass");
        let const_name = CONST_CASE.from(&pascal_name, &PASCAL_CASE).unwrap();
        assert_eq!(const_name, "MY_CLASS");
        let hungarian_member = HUNGARIAN_MEMBER.from(&const_name, &CONST_CASE).unwrap();
        assert_eq!(hungarian_member, "m_myClass");
        let hungarian_param = HUNGARIAN_PARAMETER.from(&hungarian_member, &HUNGARIAN_MEMBER).unwrap();
        assert_eq!(hungarian_param, "pMyClass");
        let snake_name = SNAKE_CASE.from(&hungarian_param, &HUNGARIAN_PARAMETER).unwrap();
        assert_eq!(snake_name, "my_class");
    }

    #[test]
    fn test_convert_type() {
        let csharp_my_class = "Tangara.MyClass";
        let rust_my_class = RUST_NAMING.convert_type(csharp_my_class, &CSHARP_NAMING).unwrap();
        assert_eq!(rust_my_class, "tangara::MyClass");
        let csharp_my_class = CSHARP_NAMING.convert_type(&rust_my_class, &RUST_NAMING).unwrap();
        assert_eq!(csharp_my_class, "Tangara.MyClass");
    }

    #[test]
    fn test_convert_package() {
        let csharp_package = "Tangara.Package";
        let rust_package = RUST_NAMING.convert_package(csharp_package, &CSHARP_NAMING).unwrap();
        assert_eq!(rust_package, "tangara-package");
        let csharp_package = CSHARP_NAMING.convert_package(&rust_package, &RUST_NAMING).unwrap();
        assert_eq!(csharp_package, "Tangara.Package");
    }
}