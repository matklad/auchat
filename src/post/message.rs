// This file is generated. Do not edit
// @generated

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_imports)]

use protobuf::Message as Message_imported_for_functions;
use protobuf::ProtobufEnum as ProtobufEnum_imported_for_functions;

#[derive(Clone,Default)]
pub struct Message {
    // message fields
    author: ::protobuf::SingularField<::std::string::String>,
    text: ::protobuf::RepeatedField<::std::string::String>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::std::cell::Cell<u32>,
}

impl Message {
    pub fn new() -> Message {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Message {
        static mut instance: ::protobuf::lazy::Lazy<Message> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Message,
        };
        unsafe {
            instance.get(|| {
                Message {
                    author: ::protobuf::SingularField::none(),
                    text: ::protobuf::RepeatedField::new(),
                    unknown_fields: ::protobuf::UnknownFields::new(),
                    cached_size: ::std::cell::Cell::new(0),
                }
            })
        }
    }

    // required string author = 1;

    pub fn clear_author(&mut self) {
        self.author.clear();
    }

    pub fn has_author(&self) -> bool {
        self.author.is_some()
    }

    // Param is passed by value, moved
    pub fn set_author(&mut self, v: ::std::string::String) {
        self.author = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_author<'a>(&'a mut self) -> &'a mut ::std::string::String {
        if self.author.is_none() {
            self.author.set_default();
        };
        self.author.as_mut().unwrap()
    }

    // Take field
    pub fn take_author(&mut self) -> ::std::string::String {
        self.author.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_author<'a>(&'a self) -> &'a str {
        match self.author.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    // repeated string text = 2;

    pub fn clear_text(&mut self) {
        self.text.clear();
    }

    // Param is passed by value, moved
    pub fn set_text(&mut self, v: ::protobuf::RepeatedField<::std::string::String>) {
        self.text = v;
    }

    // Mutable pointer to the field.
    pub fn mut_text<'a>(&'a mut self) -> &'a mut ::protobuf::RepeatedField<::std::string::String> {
        &mut self.text
    }

    // Take field
    pub fn take_text(&mut self) -> ::protobuf::RepeatedField<::std::string::String> {
        ::std::mem::replace(&mut self.text, ::protobuf::RepeatedField::new())
    }

    pub fn get_text<'a>(&'a self) -> &'a [::std::string::String] {
        &self.text
    }
}

impl ::protobuf::Message for Message {
    fn is_initialized(&self) -> bool {
        if self.author.is_none() {
            return false;
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !try!(is.eof()) {
            let (field_number, wire_type) = try!(is.read_tag_unpack());
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeLengthDelimited {
                        return ::std::result::Result::Err(::protobuf::ProtobufError::WireError("unexpected wire type".to_string()));
                    };
                    let tmp = self.author.set_default();
                    try!(is.read_string_into(tmp))
                },
                2 => {
                    try!(::protobuf::rt::read_repeated_string_into(wire_type, is, &mut self.text));
                },
                _ => {
                    let unknown = try!(is.read_unknown(wire_type));
                    self.mut_unknown_fields().add_value(field_number, unknown);
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in self.author.iter() {
            my_size += ::protobuf::rt::string_size(1, &value);
        };
        for value in self.text.iter() {
            my_size += ::protobuf::rt::string_size(2, &value);
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.author.as_ref() {
            try!(os.write_string(1, &v));
        };
        for v in self.text.iter() {
            try!(os.write_string(2, &v));
        };
        try!(os.write_unknown_fields(self.get_unknown_fields()));
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields<'s>(&'s self) -> &'s ::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields<'s>(&'s mut self) -> &'s mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn type_id(&self) -> ::std::any::TypeId {
        ::std::any::TypeId::of::<Message>()
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for Message {
    fn new() -> Message {
        Message::new()
    }

    fn descriptor_static(_: ::std::option::Option<Message>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_string_accessor(
                    "author",
                    Message::has_author,
                    Message::get_author,
                ));
                fields.push(::protobuf::reflect::accessor::make_repeated_string_accessor(
                    "text",
                    Message::get_text,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Message>(
                    "Message",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Message {
    fn clear(&mut self) {
        self.clear_author();
        self.clear_text();
        self.unknown_fields.clear();
    }
}

impl ::std::cmp::PartialEq for Message {
    fn eq(&self, other: &Message) -> bool {
        self.author == other.author &&
        self.text == other.text &&
        self.unknown_fields == other.unknown_fields
    }
}

impl ::std::fmt::Debug for Message {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

static file_descriptor_proto_data: &'static [u8] = &[
    0x0a, 0x0d, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x12,
    0x1e, 0x72, 0x75, 0x2e, 0x73, 0x70, 0x62, 0x61, 0x75, 0x2e, 0x63, 0x68, 0x61, 0x74, 0x2e, 0x63,
    0x6f, 0x6d, 0x6d, 0x6f, 0x6e, 0x73, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x63, 0x6f, 0x6c, 0x22,
    0x27, 0x0a, 0x07, 0x4d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x12, 0x0e, 0x0a, 0x06, 0x61, 0x75,
    0x74, 0x68, 0x6f, 0x72, 0x18, 0x01, 0x20, 0x02, 0x28, 0x09, 0x12, 0x0c, 0x0a, 0x04, 0x74, 0x65,
    0x78, 0x74, 0x18, 0x02, 0x20, 0x03, 0x28, 0x09, 0x4a, 0xb4, 0x01, 0x0a, 0x06, 0x12, 0x04, 0x01,
    0x00, 0x06, 0x01, 0x0a, 0x08, 0x0a, 0x01, 0x02, 0x12, 0x03, 0x01, 0x08, 0x26, 0x0a, 0x0a, 0x0a,
    0x02, 0x04, 0x00, 0x12, 0x04, 0x03, 0x00, 0x06, 0x01, 0x0a, 0x0a, 0x0a, 0x03, 0x04, 0x00, 0x01,
    0x12, 0x03, 0x03, 0x08, 0x0f, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x00, 0x02, 0x00, 0x12, 0x03, 0x04,
    0x04, 0x1f, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x04, 0x12, 0x03, 0x04, 0x04, 0x0c,
    0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x05, 0x12, 0x03, 0x04, 0x0d, 0x13, 0x0a, 0x0c,
    0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x01, 0x12, 0x03, 0x04, 0x14, 0x1a, 0x0a, 0x0c, 0x0a, 0x05,
    0x04, 0x00, 0x02, 0x00, 0x03, 0x12, 0x03, 0x04, 0x1d, 0x1e, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x00,
    0x02, 0x01, 0x12, 0x03, 0x05, 0x04, 0x1d, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x04,
    0x12, 0x03, 0x05, 0x04, 0x0c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x05, 0x12, 0x03,
    0x05, 0x0d, 0x13, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x01, 0x12, 0x03, 0x05, 0x14,
    0x18, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x03, 0x12, 0x03, 0x05, 0x1b, 0x1c,
];

static mut file_descriptor_proto_lazy: ::protobuf::lazy::Lazy<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::lazy::Lazy {
    lock: ::protobuf::lazy::ONCE_INIT,
    ptr: 0 as *const ::protobuf::descriptor::FileDescriptorProto,
};

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    unsafe {
        file_descriptor_proto_lazy.get(|| {
            parse_descriptor_proto()
        })
    }
}
