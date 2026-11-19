// Intègre l'icône (assets/icon.ico) dans kimbo.exe
fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "Kimbo");
        res.set("FileDescription", "Kimbo : sur quelle case me placer ?");
        if let Err(e) = res.compile() {
            // Ne pas bloquer la compilation si l'outil de ressources manque
            println!("cargo:warning=Icone non integree : {e}");
        }
    }
}
