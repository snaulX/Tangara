use tangara_highlevel::{Package, TypeRef, Visibility};
use tangara_highlevel::builder::*;

#[test]
#[cfg(feature = "builder")]
fn alpha_engine() {
    // Create Alpha.Window package
    let alphawindow: Package;
    {
        let builder = PackageBuilder::new("Alpha.Window");
        builder.borrow_mut().set_namespace("Alpha.Window");
        let mut type_builder = create_enum(builder.clone(), "WindowFlags")
            .bitflags();
        type_builder
            .literal("None")
            .literal("Fullscreen")
            .literal("FullscreenDesktop")
            .literal("Borderless")
            .literal("Resizable")
            .literal("Maximized");
        type_builder.build();
        let mut type_builder = create_class(builder.clone(), "Window");
        type_builder
            .set_visibility(Visibility::Public)
            .add_constructor()
                .set_visibility(Visibility::Public)
                .arg(TypeRef::from("CString"), "title")
                .arg(TypeRef::from("int"), "x")
                .arg(TypeRef::from("int"), "y")
                .arg(TypeRef::from("int"), "width")
                .arg(TypeRef::from("int"), "height")
                .arg(TypeRef::from("WindowFlags"), "state")
                .build()
            .add_property(TypeRef::from("WindowFlags"), "WindowState").build()
            .add_method("Update").build()
            .add_method("Show").build()
            .add_method("Hide").build()
            .add_method("Close").build()
            .add_method("ShouldClose")
                .return_type(TypeRef::from("bool"))
                .build()
            .add_method("GetNativeHandle")
                .return_type(TypeRef::from("Ptr"))
                .build()
            .add_method("SetSize")
                .arg(TypeRef::from("int"), "x")
                .arg(TypeRef::from("int"), "y")
                .build()
            .add_method("GetSize")
                .arg_out(TypeRef::from("int"), "x")
                .arg_out(TypeRef::from("int"), "x")
                .build()
            .add_method("SetFramebufferResizeCallback")
                .arg(TypeRef::from("Ptr"), "ptr")
                .arg(TypeRef::Fn(
                    None,
                    vec![TypeRef::from("Ptr"), TypeRef::from("int"), TypeRef::from("int")]),
                     "callback")
                .build();
        type_builder.build();
        alphawindow = builder.borrow().build();
    }
    println!("Alpha.Window package: {:?}", alphawindow);

    // Create AlphaEngine package
    let alphaengine;
    {
        let builder = PackageBuilder::new("AlphaEngine");
        builder.borrow_mut().set_namespace("Alpha.Engine");
        let mut type_builder = create_interface(builder.clone(), "ISystem");
        type_builder
            .add_method("Init").build()
            .add_method("Enable").build()
            .add_method("Disable").build();
        type_builder.build();
        alphaengine = builder.borrow().build();
    }
    println!("Alpha Engine package: {:?}", alphaengine);
}