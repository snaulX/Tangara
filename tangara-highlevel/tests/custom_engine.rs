use tangara_highlevel::{TypeRef, Visibility};
use tangara_highlevel::builder::{PackageBuilder, TypeBuilder};

#[test]
#[cfg(feature = "builder")]
fn alpha_engine() {
    // Create Alpha.Window package
    let alphawindow = PackageBuilder::new("Alpha.Window")
        .set_namespace("Alpha.Window")
        .create_enum("WindowFlags")
            .bitflags()
            .literal("None")
            .literal("Fullscreen")
            .literal("FullscreenDesktop")
            .literal("Borderless")
            .literal("Resizable")
            .literal("Maximized")
            .build()
        .create_class("Window")
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
                .build()
            .build()
        .build();
    println!("Alpha.Window package: {:?}", alphawindow);

    // Create AlphaEngine package
    let alphaengine = PackageBuilder::new("AlphaEngine")
        .set_namespace("Alpha.Engine")
        .create_interface("ISystem")
            .add_method("Init").build()
            .add_method("Enable").build()
            .add_method("Disable").build()
            .build()
        .build();
    println!("Alpha Engine package: {:?}", alphaengine);
}