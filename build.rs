use std::env;
use std::fs;
use std::path::PathBuf;
use fs_extra::dir::CopyOptions;
use fs_extra::copy_items;

// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
// Atention le nom de dossier source du project dos ce nomer 'ShowMap' !!
// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!



fn main() {
    // Inclure l'icône pour l'exécutable
    embed_resource::compile("icon.rc");

    // Récupérer le répertoire de sortie (OUT_DIR) où les binaires sont compilés
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR n'a pas été défini");
    let mut dest_path = PathBuf::from(&out_dir);

    // Modifier le chemin de destination pour pointer vers 'target/release'
    while dest_path.file_name().map_or(false, |f| f != "release") {
        dest_path.pop();
    }

    // Définir le chemin source du dossier 'assets'
    let assets_source = env::current_dir().expect("Impossible d'obtenir le répertoire courant").join("assets");
    println!("Chemin source des assets: {:?}", assets_source);

    // Vérifier si le dossier `assets` existe avant de le copier
    if !assets_source.exists() {
        panic!("Le dossier 'assets' est introuvable. Assurez-vous qu'il existe à la racine du projet.");
    }

    // Vérifier si le chemin de destination existe, sinon le créer
    if !dest_path.exists() {
        fs::create_dir_all(&dest_path).expect("Impossible de créer le répertoire de destination");
    }

    // Options de copie pour écraser les fichiers existants
    let mut options = CopyOptions::new();
    options.overwrite = true;

    // Copier le dossier 'assets' vers le chemin cible (target/release/assets)
    println!("Chemin de destination des assets: {:?}", dest_path);
    if let Err(e) = copy_items(&[assets_source], &dest_path, &options) {
        panic!("Erreur lors de la copie des fichiers d'assets : {:?}", e);
    }
}
