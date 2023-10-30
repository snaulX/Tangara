use tangara_highlevel::{typeref_by_name, TypeRef, Visibility};
use tangara_highlevel::builder::{PackageBuilder, TypeBuilder};

#[test]
#[cfg(feature = "builder")]
fn alpha_engine() {
    /*let alphaengine = Package {
        name: "AlphaEngine".to_string(),
        id: 1200984046632254291
    };*/

    // Create AlphaWindow package
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
                .arg(typeref_by_name("CString"), "title")
                .arg(typeref_by_name("int"), "x")
                .arg(typeref_by_name("int"), "y")
                .arg(typeref_by_name("int"), "width")
                .arg(typeref_by_name("int"), "height")
                .arg(typeref_by_name("WindowFlags"), "state")
                .build()
            .add_property(typeref_by_name("WindowFlags"), "WindowState").build()
            .add_method("Update").build()
            .add_method("Show").build()
            .add_method("Hide").build()
            .add_method("Close").build()
            .add_method("ShouldClose")
                .return_type(typeref_by_name("bool"))
                .build()
            .add_method("GetNativeHandle")
                .return_type(typeref_by_name("Ptr"))
                .build()
            .build()
        .build();
    println!("Alpha.Window package: {:?}", alphawindow);
}