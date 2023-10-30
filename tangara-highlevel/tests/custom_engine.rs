use tangara_highlevel::{TypeRef, Visibility};
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
                .arg(TypeRef::Name("CString".to_string()), "title")
                .arg(TypeRef::Name("int".to_string()), "x")
                .arg(TypeRef::Name("int".to_string()), "y")
                .arg(TypeRef::Name("int".to_string()), "width")
                .arg(TypeRef::Name("int".to_string()), "height")
                .arg(TypeRef::Name("WindowFlags".to_string()), "state")
                .build()
            .build()
        .build();
    println!("Alpha.Window package: {:?}", alphawindow);
}