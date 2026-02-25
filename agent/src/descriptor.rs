use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct MethodDescriptor {
    pub return_descriptor: ReturnDescriptor,
    pub parameters: Vec<FieldType>,
}

impl Display for MethodDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        for (i, parameter) in self.parameters.iter().enumerate() {
            write!(f, "{}", parameter)?;

            if i != self.parameters.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, ")")
    }
}

impl MethodDescriptor {
    pub fn new(raw: &str) -> Self {
        let end_of_parameter_descriptor = raw.find(")").unwrap();

        let mut raw_parameter_descriptor = &raw[1..end_of_parameter_descriptor];
        let parameters = if raw_parameter_descriptor.is_empty() {
            Vec::new()
        } else {
            let mut parameters = Vec::new();
            loop {
                let parameter = FieldType::new(raw_parameter_descriptor);

                if parameter.length() == raw_parameter_descriptor.len() {
                    parameters.push(parameter);
                    break;
                }

                raw_parameter_descriptor = &raw_parameter_descriptor[parameter.length()..];
                parameters.push(parameter);
            }

            parameters
        };

        let raw_return_descriptor = &raw[end_of_parameter_descriptor + 1..];

        let return_descriptor = if raw_return_descriptor == "V" {
            ReturnDescriptor::Void
        } else {
            ReturnDescriptor::FieldType(FieldType::new(raw_return_descriptor))
        };

        Self {
            return_descriptor,
            parameters,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ReturnDescriptor {
    Void,
    FieldType(FieldType),
}

#[derive(Debug, PartialEq, Clone)]
pub enum FieldType {
    Base(BaseType),
    Object { class_name: String },
    Component(Box<FieldType>),
}

impl Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldType::Base(base_type) => write!(f, "{}", base_type),
            FieldType::Object { class_name } => write!(f, "{}", class_name),
            FieldType::Component(field_type) => write!(f, "{}", field_type),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum BaseType {
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Short,
    Boolean,
}

impl Display for BaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BaseType::Byte => write!(f, "byte"),
            BaseType::Char => write!(f, "char"),
            BaseType::Double => write!(f, "double"),
            BaseType::Float => write!(f, "float"),
            BaseType::Int => write!(f, "int"),
            BaseType::Long => write!(f, "long"),
            BaseType::Short => write!(f, "short"),
            BaseType::Boolean => write!(f, "boolean"),
        }
    }
}

impl FieldType {
    fn new(raw: &str) -> Self {
        match &raw[0..1] {
            "B" => Self::Base(BaseType::Byte),
            "C" => Self::Base(BaseType::Char),
            "D" => Self::Base(BaseType::Double),
            "F" => Self::Base(BaseType::Float),
            "I" => Self::Base(BaseType::Int),
            "J" => Self::Base(BaseType::Long),
            "S" => Self::Base(BaseType::Short),
            "Z" => Self::Base(BaseType::Boolean),
            "L" => Self::Object {
                class_name: raw[1..raw.find(';').unwrap()].to_string(),
            },
            "[" => Self::Component(Box::new(Self::new(&raw[1..]))),
            _ => panic!("unknown field type: {raw}"),
        }
    }

    fn length(&self) -> usize {
        match self {
            FieldType::Base(_) => 1,
            FieldType::Object { class_name } => class_name.len() + 2,
            FieldType::Component(field_type) => field_type.length() + 1,
        }
    }
}
