use tangara_highlevel::Visibility;
use tangara_highlevel::builder::PackageBuilder;

#[test]
#[cfg(feature = "builder")]
fn alpha_engine() {
    /*let alphaengine = Package {
        name: "AlphaEngine".to_string(),
        id: 1200984046632254291
    };*/

    // Create AlphaWindow
    let alphawindow = PackageBuilder::new("AlphaWindow")
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
            .build()
        .build();
    println!("Alpha window id: {:?}", alphawindow);
}