#![windows_subsystem = "windows"]

use eframe::{egui};
use eframe::{NativeOptions, run_native};
use image::GenericImageView;
use native_dialog::FileDialog as NativeFileDialog;
use rfd::FileDialog as RfdFileDialog;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;
use std::io::BufWriter;
use std::fs::File;
use native_dialog::MessageDialog;
use native_dialog::MessageType;
use pdf_creation::Mm;
use pdf_creation::BuiltinFont;
use lopdf::StringFormat; // Ajout de StringFormat pour l'utiliser directement
use ttf_parser::Face; // Utilisé pour lire les largeurs des glyphes
use lopdf::Object;
use crate::pdf_modification::Document;
use eframe::epaint::TextureHandle; // Assurez-vous que cela est inclus
use std::process::Command;
use std::fs;



//  MAcro Dictionary
use lopdf::dictionary;


mod pdf_creation {
    pub use printpdf::*;
}

mod pdf_modification {
    pub use lopdf::{Document, Object, Stream};
    pub use lopdf::dictionary; // Ajout pour exposer la macro dictionary!
}








// Structure pour représenter un fichier PDF
#[derive(Clone)]
struct PdfFile {
    path: PathBuf,
    selected: bool,
}

// Enumération pour les différents formats de PDF
#[derive(PartialEq)] // Ajout de PartialEq
#[derive(Clone, Copy, Debug)]
enum PDFFormat {
    None,
    A4,
    A4Landscape,
    A5,
    A5Landscape,
    USLetter,
    USLetterLandscape,
    Automatique,
}

// Fonction principale
fn main() {
    // Récupérer le répertoire courant
    let current_dir = std::env::current_dir().expect("Impossible de récupérer le répertoire courant");
    println!("Répertoire courant : {}", current_dir.display());

    // Construire le chemin relatif vers l'icône
    let icon_path = current_dir.join("assets").join("ICON").join("ICON.png");
    println!("Chemin de l'icône : {}", icon_path.display());

    // Charger l'icône
    let (icon_width, icon_height, icon_rgba) = load_icon(&icon_path).expect("Erreur lors du chargement de l'icône");

    // Configurer les options natives avec l'icône
    let options = NativeOptions {
        icon_data: Some(eframe::IconData {
            rgba: icon_rgba,
            width: icon_width,
            height: icon_height,
        }),
        ..Default::default()
    };

    // Lancer l'application
    run_native(
        "Sélecteur de PDF et Prévisualisation",
        options,
        Box::new(|_cc| Box::new(AppState::default())),
    );
}






// Structure de l'état de l'application
struct AppState {
    pdf_files: Arc<Mutex<Vec<PdfFile>>>,
    root_folder: Arc<Mutex<Option<PathBuf>>>,
    selected_format: Arc<Mutex<PDFFormat>>, // Format sélectionné
    annotation_position: Arc<Mutex<(f64, f64)>>, // Position de l'annotation
    custom_text: Arc<Mutex<String>>,    // Text a ajouter
    pdf_files_info: Arc<Mutex<Vec<PdfFile>>>, // Nouvel attribut pour stocker les informations sur les fichiers PDF trouvés
    custom_text_info: Arc<Mutex<String>>,  // Texte personnalisé copier
    apply_to_all_pages: Arc<Mutex<bool>>, // True pour toutes les pages, False pour la première page uniquement
    show_no_root_popup: Arc<Mutex<bool>>,
    show_no_pdf_popup: Arc<Mutex<bool>>,
    show_pdf_open: Arc<Mutex<bool>>,
    show_success_icon: Arc<Mutex<bool>>, // Indique si l'icône de succès est visible
    success_texture: Option<TextureHandle>, // Texture pour l'image de succès
    show_annotation_removal_popup: Arc<Mutex<bool>>, // État de la popup
    annotation_removal_count: Arc<Mutex<usize>>,    // Compteur des fichiers traités
    repaired_file_count: Arc<Mutex<usize>>, // Compteur pour les fichiers réparés
    show_repair_popup: Arc<Mutex<bool>>,    // Indicateur pour afficher le popup de réparation
    selected_color: egui::Color32, // Ajouter ce champ pour la couleur
    selected_color_copie_1: egui::Color32, // Ajouter ce champ pour la couleur
    selected_color_copie_2: egui::Color32, // Ajouter ce champ pour la couleur
}

// Valeurs par défaut pour `AppState`
impl Default for AppState {
    fn default() -> Self {
        AppState {
            pdf_files: Arc::new(Mutex::new(Vec::new())),
            root_folder: Arc::new(Mutex::new(None)),
            selected_format: Arc::new(Mutex::new(PDFFormat::Automatique)),
            annotation_position: Arc::new(Mutex::new((10.0, 10.0))),
            custom_text: Arc::new(Mutex::new("Crée par 00MY00".to_string())), // Text par défault
            pdf_files_info: Arc::new(Mutex::new(Vec::new())), // Initialisation de pdf_files_info
            custom_text_info: Arc::new(Mutex::new("Crée par 00MY00".to_string())), // Initialisation de custom_text_info avec le même texte
            apply_to_all_pages: Arc::new(Mutex::new(false)), // Par défaut : appliquer sur toutes les pages
            show_no_root_popup: Arc::new(Mutex::new(false)),
            show_no_pdf_popup: Arc::new(Mutex::new(false)),
            show_pdf_open: Arc::new(Mutex::new(false)),
            show_success_icon: Arc::new(Mutex::new(false)),
            success_texture: None, // Initialisé comme `None`
            show_annotation_removal_popup: Arc::new(Mutex::new(false)),
            annotation_removal_count: Arc::new(Mutex::new(0)),
            repaired_file_count: Arc::new(Mutex::new(0)), // Ajout
            show_repair_popup: Arc::new(Mutex::new(false)), // Ajout
            selected_color: egui::Color32::BLACK, // Couleur par défaut
            selected_color_copie_1: egui::Color32::BLACK, // Couleur par défaut    
            selected_color_copie_2: egui::Color32::BLACK, // Couleur par défaut            
        }
    }
}





// Couleurs
const CUSTOM_TEXT_COLOR_FAFAFA: egui::Color32 = egui::Color32::from_rgb(0xFA, 0xFA, 0xFA); // Blanc
const CUSTOM_TEXT_COLOR_0093D3: egui::Color32 = egui::Color32::from_rgb(0x00, 0x93, 0xD3); // Bleu clair
const CUSTOM_TEXT_COLOR_0B84B8: egui::Color32 = egui::Color32::from_rgb(0x0B, 0x84, 0xB8); // Bleu foncé
const CUSTOM_TEXT_COLOR_067DF4: egui::Color32 = egui::Color32::from_rgb(0x06, 0x7d, 0xf4); // Bleu foncé +
const CUSTOM_TEXT_COLOR_000000: egui::Color32 = egui::Color32::from_rgb(0x00, 0x00, 0x00); // Noire



fn setup_custom_theme(ctx: &egui::Context) {
    // Créez une instance de `Visuals` personnalisée.
    let mut visuals = egui::Visuals::dark(); // Ou `egui::Visuals::dark()` pour un thème sombre.

    // Couleurs pour les boutons actifs (état enfoncé ou cliqué).
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0x66, 0x99, 0xCC); // Bleu clair.
    visuals.widgets.active.rounding = egui::Rounding::same(5.0); // Coins arrondis.
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(0x33, 0x66, 0x99)); // Contour bleu.

    // Couleurs pour les boutons survolés (hovered).
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(0x13, 0x32, 0x78); // Bleu plus foncé (valeur corrigée).
    visuals.widgets.hovered.rounding = egui::Rounding::same(5.0); // Coins arrondis.
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(0x66, 0x99, 0xCC)); // Contour bleu clair.

    // Couleurs pour les boutons inactifs (par défaut).
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(0x86, 0x86, 0x86); // Gris foncé.
    visuals.widgets.inactive.rounding = egui::Rounding::same(5.0); // Coins arrondis.
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0x70, 0x70, 0x70)); // Contour gris.

    // Couleur de texte par défaut pour tous les widgets.
    visuals.override_text_color = Some(egui::Color32::from_rgb(0xFA, 0xFA, 0xFA)); // Blanc (valeur corrigée).

    // Appliquer le style global au contexte `egui`.
    ctx.set_visuals(visuals);
}












// Fonction pour charger une icône à partir d'une image
fn load_icon(icon_path: &Path) -> Option<(u32, u32, Vec<u8>)> {
    let image = image::open(icon_path).ok()?; // Charger l'image avec la bibliothèque `image`
    let (width, height) = image.dimensions(); // Obtenir les dimensions de l'image
    let rgba = image.into_rgba8().into_raw(); // Convertir en données RGBA
    Some((width, height, rgba))
}





// Fonction pour chercher les fichiers PDF de manière récursive
fn find_pdfs(root: &Path) -> Vec<PdfFile> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                if e.path().extension().map_or(false, |ext| ext == "pdf") {
                    Some(PdfFile {
                        path: e.path().to_path_buf(),
                        selected: true, // Par défaut, tous les fichiers sont sélectionnés
                    })
                } else {
                    None
                }
            })
        })
        .collect()
}


// Fonction pour obtenir les dimensions en fonction du format
fn get_dimensions_by_format(format: PDFFormat) -> Option<(pdf_creation::Mm, pdf_creation::Mm)> {
    match format {
        PDFFormat::A4 => Some((pdf_creation::Mm(210.0), pdf_creation::Mm(297.0))),
        PDFFormat::A4Landscape => Some((pdf_creation::Mm(297.0), pdf_creation::Mm(210.0))), // Inversé
        PDFFormat::A5 => Some((pdf_creation::Mm(148.0), pdf_creation::Mm(210.0))),
        PDFFormat::A5Landscape => Some((pdf_creation::Mm(210.0), pdf_creation::Mm(148.0))), // Inversé
        PDFFormat::USLetter => Some((pdf_creation::Mm(215.9), pdf_creation::Mm(279.4))),
        PDFFormat::USLetterLandscape => Some((pdf_creation::Mm(279.4), pdf_creation::Mm(215.9))), // Inversé
        PDFFormat::Automatique => Some((pdf_creation::Mm(210.0), pdf_creation::Mm(297.0))),
        _ => None,
    }
}


// Fonction pour réduire les dimensions de 50%
fn scale_dimensions_by_half(dimensions: (Mm, Mm)) -> (Mm, Mm) {
    // Réduire à 50 % de la taille originale
    (Mm(dimensions.0.0 * 0.5), Mm(dimensions.1.0 * 0.5))
}

// Fonction pour restaurer les dimensions d'origine à partir des dimensions réduites
fn restore_original_dimensions(dimensions: (Mm, Mm)) -> (Mm, Mm) {
    (Mm(dimensions.0.0 * 2.0), Mm(dimensions.1.0 * 2.0))
}





// Fonction pour tenter une réparation avec QPDF et recharger le fichier
fn repair_and_reload_pdf(file_path: &Path) -> Result<Document, String> {
    // Obtenir le répertoire courant pour construire le chemin complet vers QPDF
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Erreur lors de la récupération du répertoire courant : {}", e))?;
    let qpdf_path = current_dir
        .join("assets")
        .join("PDF_Tools")
        .join("qpdf 11.9.1")
        .join("bin")
        .join("qpdf.exe");

    // Vérifier si le chemin vers QPDF existe
    if !qpdf_path.exists() {
        return Err(format!(
            "Erreur : QPDF introuvable au chemin spécifié : {}",
            qpdf_path.display()
        ));
    }

    // Exécuter la réparation avec QPDF, en écrasant le fichier d'origine
    let output = Command::new(qpdf_path)
        .args(&[
            "--replace-input", // Option pour écraser directement le fichier d'entrée
            file_path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Erreur lors de l'exécution de QPDF : {}", e))?;

    // Afficher les logs pour le débogage
    println!("QPDF stdout : {}", String::from_utf8_lossy(&output.stdout));
    println!("QPDF stderr : {}", String::from_utf8_lossy(&output.stderr));

    if !output.status.success() {
        return Err(format!(
            "Échec de la réparation avec QPDF : {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Recharger le fichier réparé
    Document::load(file_path).map_err(|e| format!("Erreur lors du chargement du fichier réparé : {}", e))
}




fn clear_backup_files(root_path: &Path) -> Result<(), String> {
    // Vérifier si le dossier racine est valide
    if !root_path.is_dir() {
        return Err(format!("Le chemin spécifié n'est pas un dossier valide : {}", root_path.display()));
    }

    // Chemin du dossier BackUP
    let backup_dir = root_path.join("BackUP");

    // Créer le dossier BackUP s'il n'existe pas
    if !backup_dir.exists() {
        println!("Le dossier BackUP n'existe pas. Tentative de création...");
        fs::create_dir_all(&backup_dir)
            .map_err(|e| format!("Erreur lors de la création du dossier BackUP : {}", e))?;
        println!("Dossier BackUP créé avec succès à l'emplacement : {}", backup_dir.display());
    } else {
        println!("Le dossier BackUP existe déjà à l'emplacement : {}", backup_dir.display());
    }

    // Rechercher les fichiers avec la terminaison `.~qpdf-orig`
    let entries = fs::read_dir(root_path).map_err(|e| format!("Erreur lors de la lecture du dossier racine : {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Erreur lors de l'accès à une entrée : {}", e))?;
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "~qpdf-orig") {
            // Déplacer le fichier dans le dossier BackUP
            let new_backup_path = backup_dir.join(path.file_name().unwrap());
            fs::rename(&path, &new_backup_path)
                .map_err(|e| format!("Erreur lors du déplacement du fichier {} : {}", path.display(), e))?;
            println!(
                "Fichier de sauvegarde déplacé : {} -> {}",
                path.display(),
                new_backup_path.display()
            );
        }
    }

    println!("Nettoyage des fichiers de sauvegarde terminé.");
    Ok(())
}









fn repair_pdf_with_xpdf(input_path: &Path, output_path: &Path) -> Result<(), String> {
    // Chemin complet vers pdftocairo
    let pdftocairo_path = ".\\assets\\PDF_Tools\\XpdfReader-win64-4.05.exe";

    // Construire la commande Xpdf
    let status = Command::new(pdftocairo_path) // Utilise le chemin complet
        .arg("-pdf")
        .arg(input_path)
        .arg(output_path)
        .status();

    match status {
        Ok(exit_status) if exit_status.success() => {
            println!("Réparation réussie : {}", output_path.display());
            Ok(())
        }
        Ok(exit_status) => {
            Err(format!(
                "Xpdf a échoué avec le code de sortie : {}",
                exit_status.code().unwrap_or(-1)
            ))
        }
        Err(e) => Err(format!("Erreur lors de l'exécution de Xpdf : {}", e)),
    }
}









/// Calcule les dimensions d'une textbox en fonction du texte et de la taille de la police.
/// 
/// # Arguments
/// * `text` - Le texte dont on veut calculer la dimension.
/// * `font_size` - Taille de la police en points (pt).
/// 
/// # Retourne
/// * (largeur_mm, hauteur_mm) - Les dimensions de la textbox en millimètres.
fn calculate_textbox_dimensions(text: &str, font_size: f64) -> Option<(f64, f64)> {
    // Conversion des points en millimètres (1 point = 0.352778 mm)
    let pt_to_mm = 0.352778;

    // Hauteur de la ligne (en points), généralement la taille de la police
    let line_height_pt = font_size;
    let line_height_mm = line_height_pt * pt_to_mm;

    // Largeur totale du texte (en points)
    let avg_char_width_pt = font_size * 0.6; // Supposition : largeur moyenne d'un caractère est 60 % de la taille de la police
    let text_width_pt = avg_char_width_pt * text.chars().count() as f64;

    let text_width_mm = text_width_pt * pt_to_mm;

    // Calculer la hauteur totale en fonction du nombre de lignes (ici, on suppose une seule ligne)
    let num_lines = 1; // Vous pouvez adapter cela si le texte est sur plusieurs lignes
    let total_height_mm = line_height_mm * num_lines as f64;

    Some((text_width_mm, total_height_mm))
}






/// Extrait les dimensions des pages d'un document PDF, affiche les boîtes trouvées, et retourne le format détecté.
fn get_page_dimensions(pdf_path: &Path) -> Option<(String, (f64, f64))> {
    // Charger le document PDF
    let doc = Document::load(pdf_path).ok()?;

    // Obtenir la liste des pages
    let pages = doc.get_pages();
    println!("Nombre de pages dans le fichier : {}", pages.len());

    // Obtenir la première page (clé et ID)
    let (first_page_number, first_page_id) = pages.into_iter().next()?;

    // Récupérer le dictionnaire de la première page
    if let Ok(Object::Dictionary(page_dict)) = doc.get_object(first_page_id) {
        println!("Dictionnaire de la première page trouvé (page {})", first_page_number);

        // Boîtes à vérifier
        let box_keys: [&[u8]; 5] = [b"MediaBox", b"CropBox", b"ArtBox", b"BleedBox", b"TrimBox"];

        // Parcourir les boîtes
        for &box_key in &box_keys {
            if let Some((width, height)) = extract_box_dimensions(&page_dict, box_key) {
                let format = detect_page_format(width, height);
                println!(
                    "Boîte : {} - Dimensions finales : largeur = {:.2} mm, hauteur = {:.2} mm - Format détecté : {}",
                    String::from_utf8_lossy(box_key),
                    width,
                    height,
                    format
                );
                // Retourne la première boîte trouvée et le format
                return Some((format, (width, height)));
            }
        }
    } else {
        println!("Aucun dictionnaire trouvé pour la première page.");
    }

    println!("Erreur : Impossible de détecter les dimensions du fichier {}", pdf_path.display());
    None
}

/// Extrait les dimensions d'une boîte spécifique dans un dictionnaire.
fn extract_box_dimensions(page_dict: &lopdf::Dictionary, box_key: &[u8]) -> Option<(f64, f64)> {
    if let Ok(Object::Array(box_array)) = page_dict.get(box_key) {
        println!("Clé trouvée : {}", String::from_utf8_lossy(box_key));
        println!("Contenu brut de la boîte : {:?}", box_array);

        if box_array.len() == 4 {
            // Vérifier si les deux premières valeurs sont des zéros
            let is_zero_based = match (box_array.get(0), box_array.get(1)) {
                (Some(Object::Real(x1)), Some(Object::Real(y1))) => *x1 == 0.0 && *y1 == 0.0,
                (Some(Object::Integer(x1)), Some(Object::Integer(y1))) => *x1 == 0 && *y1 == 0,
                _ => false,
            };

            if is_zero_based {
                // Récupérer les dimensions finales (x2, y2)
                if let (
                    Some(Object::Real(x2)),
                    Some(Object::Real(y2)),
                ) = (
                    box_array.get(2),
                    box_array.get(3),
                ) {
                    let width = round_to_two_decimals(*x2 as f64 * 0.352778); // Conversion en mm
                    let height = round_to_two_decimals(*y2 as f64 * 0.352778); // Conversion en mm
                    return Some((width, height));
                }

                if let (
                    Some(Object::Integer(x2)),
                    Some(Object::Integer(y2)),
                ) = (
                    box_array.get(2),
                    box_array.get(3),
                ) {
                    let width = round_to_two_decimals((*x2 as f64) * 0.352778); // Conversion en mm
                    let height = round_to_two_decimals((*y2 as f64) * 0.352778); // Conversion en mm
                    return Some((width, height));
                }
            }
        }
    }

    None
}

/// Détecte le format de la page en fonction de ses dimensions (en millimètres).
fn detect_page_format(width: f64, height: f64) -> String {
    let a4_dimensions = (210.0, 297.0);
    let a4_landscape = (297.0, 210.0);
    let a5_dimensions = (148.0, 210.0);
    let a5_landscape = (210.0, 148.0);
    let us_letter = (215.9, 279.4);
    let us_letter_landscape = (279.4, 215.9);

    if (width - a4_dimensions.0).abs() < 1.0 && (height - a4_dimensions.1).abs() < 1.0 {
        "A4".to_string()
    } else if (width - a4_landscape.0).abs() < 1.0 && (height - a4_landscape.1).abs() < 1.0 {
        "A4 (Paysage)".to_string()
    } else if (width - a5_dimensions.0).abs() < 1.0 && (height - a5_dimensions.1).abs() < 1.0 {
        "A5".to_string()
    } else if (width - a5_landscape.0).abs() < 1.0 && (height - a5_landscape.1).abs() < 1.0 {
        "A5 (Paysage)".to_string()
    } else if (width - us_letter.0).abs() < 1.0 && (height - us_letter.1).abs() < 1.0 {
        "US Letter".to_string()
    } else if (width - us_letter_landscape.0).abs() < 1.0 && (height - us_letter_landscape.1).abs() < 1.0 {
        "US Letter (Paysage)".to_string()
    } else {
        format!("Custom {:.2} x {:.2} mm", width, height)
    }
}


/// Fonction pour arrondir à deux décimales.
fn round_to_two_decimals(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}












// Ajuster les dimensions d'une position en fonction du format d'origine et du format cible
fn adjust_position(
    preview_dimensions: (f64, f64),   // Dimensions de la prévisualisation (A4 simulée, en mm)
    original_dimensions: (f64, f64), // Dimensions réelles (A4 en mm)
    target_dimensions: (f64, f64),   // Dimensions réelles détectées
    position: (f64, f64),            // Position d'annotation issue de la prévisualisation
) -> (f64, f64) {
    if preview_dimensions.0 == 0.0 || preview_dimensions.1 == 0.0 {
        println!("Dimensions de prévisualisation invalides : {:?}", preview_dimensions);
        return position; // Ne pas ajuster si les dimensions de prévisualisation sont invalides
    }

    // Étape 1 : Adapter les positions de la prévisualisation à un A4 réel
    let scale_x_preview = original_dimensions.0 / preview_dimensions.0;
    let scale_y_preview = original_dimensions.1 / preview_dimensions.1;

    let adjusted_position_to_a4 = (
        position.0 * scale_x_preview,
        position.1 * scale_y_preview,
    );

    println!(
        "Position adaptée au A4 réel : ({:.2}, {:.2}), échelles de prévisualisation = (x: {:.2}, y: {:.2})",
        adjusted_position_to_a4.0, adjusted_position_to_a4.1, scale_x_preview, scale_y_preview
    );

    // Étape 2 : Ajuster les positions du A4 réel au format détecté (ex: A4 Paysage, A5, etc.)
    let scale_x_target = target_dimensions.0 / original_dimensions.0;
    let scale_y_target = target_dimensions.1 / original_dimensions.1;

    let adjusted_position_to_target = (
        adjusted_position_to_a4.0 * scale_x_target,
        adjusted_position_to_a4.1 * scale_y_target,
    );

    println!(
        "Position ajustée pour le format cible : ({:.2}, {:.2}), échelles du A4 au format cible = (x: {:.2}, y: {:.2})",
        adjusted_position_to_target.0, adjusted_position_to_target.1, scale_x_target, scale_y_target
    );

    adjusted_position_to_target
}







fn ajouter_police(pdf_files: &Vec<PdfFile>, roboto_font_path: &str, font_name: &str) {
    use std::fs;
    use pdf_modification::{Document, Object, Stream, dictionary};

    println!("Début de l'ajout de la police aux fichiers PDF.");

    for pdf_file in pdf_files {
        if let Some(output_path) = pdf_file.path.to_str() {
            let mut doc = match Document::load(output_path) {
                Ok(doc) => doc,
                Err(e) => {
                    eprintln!("Erreur lors du chargement du fichier PDF {}: {}", output_path, e);
                    continue;
                }
            };

            let roboto_data = match fs::read(roboto_font_path) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Erreur lors de la lecture du fichier de police : {}", e);
                    continue;
                }
            };

            

            let font_already_added = doc.objects.iter().any(|(_, obj)| {
                if let Ok(dict) = obj.as_dict() {
                    if let Ok(base_font) = dict.get(b"BaseFont").and_then(|base| base.as_name()) {
                        // Convertir `font_name` en bytes pour la comparaison
                        return base_font == font_name.as_bytes();
                    }
                }
                false
            });
            

            if font_already_added {
                println!(
                    "La police {} est déjà présente dans le fichier : {}",
                    String::from_utf8_lossy(font_name.as_bytes()), // Convertir font_name en &[u8]
                    output_path
                );
                continue;
            }
            

            let font_object_id = (doc.max_id + 1, 0);
            doc.max_id += 1;

            let font_stream = Object::Stream(Stream::new(
                dictionary! {
                    b"Type" => Object::Name(b"Font".to_vec()),
                    b"Subtype" => Object::Name(b"TrueType".to_vec()),
                    b"BaseFont" => Object::Name(font_name.as_bytes().to_vec()),
                    b"Encoding" => Object::Name(b"WinAnsiEncoding".to_vec()),
                },
                roboto_data,
            ));

            doc.objects.insert(font_object_id, font_stream);

            for (page_number, page_id) in doc.get_pages() {
                println!("Ajout des polices sur la page {}", page_number);

                if let Ok(Object::Dictionary(ref mut page_dict)) = doc.get_object_mut(page_id) {
                    let resources = match page_dict.get_mut(b"Resources") {
                        Ok(Object::Dictionary(resources)) => resources,
                        _ => {
                            let new_resources = dictionary! {};
                            page_dict.set(b"Resources", Object::Dictionary(new_resources.clone()));
                            page_dict.get_mut(b"Resources").unwrap().as_dict_mut().unwrap()
                        }
                    };

                    let fonts = match resources.get_mut(b"Font") {
                        Ok(Object::Dictionary(fonts)) => fonts,
                        _ => {
                            let new_fonts = dictionary! {};
                            resources.set(b"Font", Object::Dictionary(new_fonts.clone()));
                            resources.get_mut(b"Font").unwrap().as_dict_mut().unwrap()
                        }
                    };

                    fonts.set(font_name, Object::Reference(font_object_id));
                    println!("Police ajoutée à la page {}.", page_number);
                }
            }

            println!("Police ajoutée avec succès au fichier PDF : {}", output_path);

            if let Err(e) = doc.save(output_path) {
                eprintln!("Erreur lors de la sauvegarde du fichier PDF {} : {}", output_path, e);
            }
        } else {
            eprintln!("Erreur : Impossible de convertir le chemin en chaîne pour le fichier {}",
                       pdf_file.path.display());
        }
    }

    println!("Fin de l'ajout de la police aux fichiers PDF.");
}












/// État possible de la police dans un PDF
#[derive(Debug)]
pub enum PoliceStatus {
    PresenteEtValide,
    NonPresente,
    NonIntegree,
    FluxVideOuCorrompu,
    NonEnregistree, 
    ErreurExtraction(FluxStatus), // Nouvelle variante pour indiquer l'échec de l'extraction
    Erreur(String),
}

/// Statut détaillé pour les problèmes rencontrés avec le flux de la police
#[derive(Debug)]
pub enum FluxStatus {
    InvalideGlyphes(String), // Aucun glyphe valide trouvé
    Illisible(String),       // Flux illisible ou corrompu
}

/// Informations sur les polices trouvées dans un PDF
#[derive(Debug)]
pub struct PoliceInfo {
    pub status: PoliceStatus,       // Statut global du PDF par rapport à la police recherchée
    pub polices_existantes: Vec<PoliceDetail>, // Liste des détails pour chaque police
}

/// Détails d'une police spécifique trouvée dans un PDF
#[derive(Debug)]
pub struct PoliceDetail {
    pub nom: String,         // Nom de la police
    pub is_integrated: bool, // Indique si la police est correctement intégrée
    pub erreur: Option<String>, // Erreur rencontrée, si applicable
}




pub fn verifier_polices(
    pdf_path: &str,
    police_a_verifier: &str,
) -> Result<PoliceInfo, Box<dyn std::error::Error>> {
    use pdf_modification::{Document, Object};

    let mut polices_details = Vec::new();

    let doc = Document::load(pdf_path)
        .map_err(|e| format!("Erreur lors du chargement du fichier : {}", e))?;

    for (obj_id, obj) in &doc.objects {
        if let Ok(dictionary) = obj.as_dict() {
            if dictionary.has(b"Type") && dictionary.get(b"Type")?.as_name()? == b"Font" {
                if let Ok(base_font) = dictionary.get(b"BaseFont").and_then(|obj| obj.as_name()) {
                    let base_font_str = std::str::from_utf8(base_font).unwrap_or_default();

                    // Vérifier si la police est intégrée ou non
                    let mut is_integrated = false;
                    let mut erreur = None;
                    if dictionary.has(b"FontFile") || dictionary.has(b"FontFile2") || dictionary.has(b"FontFile3") {
                        if let Ok(font_stream_id) = dictionary.get(b"FontFile")
                            .or_else(|_| dictionary.get(b"FontFile2"))
                            .or_else(|_| dictionary.get(b"FontFile3"))
                        {
                            if let Object::Stream(stream) = doc.get_object(font_stream_id.as_reference().unwrap())? {
                                if stream.content.is_empty() {
                                    erreur = Some("Flux de police vide".to_string());
                                } else {
                                    // Valider le flux de police
                                    if let Some(flux_status) = verifier_flux_de_police(&stream.content) {
                                        erreur = Some(format!("Erreur dans le flux : {:?}", flux_status));
                                    } else {
                                        is_integrated = true;
                                    }
                                }
                            } else {
                                erreur = Some("Flux de police introuvable".to_string());
                            }
                        }
                    } else {
                        erreur = Some("La police n'est pas intégrée".to_string());
                    }

                    polices_details.push(PoliceDetail {
                        nom: base_font_str.to_string(),
                        is_integrated,
                        erreur,
                    });
                }
            }
        }
    }

    let status = if polices_details.iter().any(|p| p.nom == police_a_verifier && p.is_integrated) {
        PoliceStatus::PresenteEtValide
    } else if polices_details.iter().any(|p| p.nom == police_a_verifier) {
        PoliceStatus::NonIntegree
    } else {
        PoliceStatus::NonPresente
    };

    Ok(PoliceInfo {
        status,
        polices_existantes: polices_details,
    })
}



/// Fonction pour vérifier la validité du flux de police avec des erreurs détaillées
fn verifier_flux_de_police(content: &[u8]) -> Option<FluxStatus> {
    use ttf_parser::Face;

    if let Ok(face) = Face::from_slice(content, 0) {
        if face.number_of_glyphs() > 0 {
            println!(
                "La police contient {} glyphes valides.",
                face.number_of_glyphs()
            );
            None // Aucun problème détecté
        } else {
            Some(FluxStatus::InvalideGlyphes(
                "La police ne contient aucun glyphe valide.".to_string(),
            ))
        }
    } else {
        Some(FluxStatus::Illisible(
            "Impossible de lire le flux de police (peut-être corrompu).".to_string(),
        ))
    }
}





























fn remplacer_polices_non_integrees(
    pdf_path: &Path,
    roboto_font_path: &str,
    font_name: &str,
    police_a_remplacer: &str, // Nouvelle entrée pour indiquer quelle police remplacer
) -> Result<(), String> {
    use std::fs;
    use pdf_modification::{Document, Object, Stream, dictionary};
    use ttf_parser::Face;

    println!("Traitement du fichier : {}", pdf_path.display());

    // Charger le document PDF
    let mut doc = Document::load(pdf_path)
        .map_err(|e| format!("Erreur lors du chargement du fichier PDF : {}", e))?;

    // Lire la police Roboto
    let roboto_data = fs::read(roboto_font_path)
        .map_err(|e| format!("Erreur lors de la lecture du fichier de police : {}", e))?;

    // Charger les informations de la police avec ttf-parser
    let face = Face::from_slice(&roboto_data, 0)
        .map_err(|e| format!("Erreur lors de l'analyse du fichier de police : {:?}", e))?;

    // Construire le dictionnaire `/Widths` à partir des informations de glyphes
    let widths: Vec<Object> = (32..=255)
        .map(|code_point| {
            if let Some(char_code) = char::from_u32(code_point as u32) {
                face.glyph_index(char_code)
                    .and_then(|id| face.glyph_hor_advance(id))
                    .map(|advance| Object::Integer(advance as i64))
                    .unwrap_or(Object::Integer(500)) // Valeur par défaut
            } else {
                Object::Integer(500) // Valeur par défaut si la conversion échoue
            }
        })
        .collect();

    let ascent = face.ascender() as i64;
    let descent = face.descender() as i64;
    let bbox = face.global_bounding_box();

    // Ajouter la police Roboto au document
    let font_object_id = (doc.max_id + 1, 0);
    let font_descriptor_id = (doc.max_id + 2, 0);
    doc.max_id += 2;

    // Définir un dictionnaire FontDescriptor
    let font_descriptor = dictionary! {
        b"Type" => Object::Name(b"FontDescriptor".to_vec()),
        b"FontName" => Object::Name(font_name.as_bytes().to_vec()),
        b"Ascent" => Object::Integer(ascent),
        b"Descent" => Object::Integer(descent),
        b"CapHeight" => Object::Integer(ascent),
        b"ItalicAngle" => Object::Integer(0),
        b"StemV" => Object::Integer(80),
        b"FontBBox" => Object::Array(vec![
            Object::Integer(bbox.x_min as i64),
            Object::Integer(bbox.y_min as i64),
            Object::Integer(bbox.x_max as i64),
            Object::Integer(bbox.y_max as i64),
        ]),
        b"FontFile2" => Object::Reference(font_object_id),
    };

    doc.objects
        .insert(font_descriptor_id, Object::Dictionary(font_descriptor));

    let font_stream = Object::Stream(Stream::new(
        dictionary! {
            b"Type" => Object::Name(b"Font".to_vec()),
            b"Subtype" => Object::Name(b"TrueType".to_vec()),
            b"BaseFont" => Object::Name(font_name.as_bytes().to_vec()),
            b"Encoding" => Object::Name(b"WinAnsiEncoding".to_vec()),
            b"FirstChar" => Object::Integer(32),
            b"LastChar" => Object::Integer(255),
            b"Widths" => Object::Array(widths.clone()),
            b"FontDescriptor" => Object::Reference(font_descriptor_id),
        },
        roboto_data,
    ));

    doc.objects.insert(font_object_id, font_stream);

    // Identifier les polices problématiques et remplacer les références
    let mut polices_a_remplacer = vec![];

    for (obj_id, obj) in &doc.objects {
        if let Ok(dictionary) = obj.as_dict() {
            if dictionary.has(b"Type") && dictionary.get(b"Type").unwrap().as_name().unwrap_or(b"") == b"Font" {
                if let Ok(base_font) = dictionary.get(b"BaseFont").and_then(|base| base.as_name()) {
                    if base_font == police_a_remplacer.as_bytes() {
                        println!(
                            "Police cible détectée pour remplacement : {} (ID : {:?})",
                            police_a_remplacer,
                            obj_id
                        );
                        polices_a_remplacer.push(*obj_id);
                    }
                }
            }
        }
    }

    // Remplacer les polices problématiques
    for font_id in polices_a_remplacer {
        if let Ok(dict) = doc.get_object_mut(font_id).and_then(|obj| obj.as_dict_mut()) {
            dict.set(b"Subtype", Object::Name(b"TrueType".to_vec()));
            dict.set(b"BaseFont", Object::Name(font_name.as_bytes().to_vec()));
            dict.set(b"Encoding", Object::Name(b"WinAnsiEncoding".to_vec()));
            dict.set(b"Widths", Object::Array(widths.clone()));
            dict.set(b"FontDescriptor", Object::Reference(font_descriptor_id));
        }
    }

    // Sauvegarder le document modifié
    doc.save(pdf_path)
        .map_err(|e| format!("Erreur lors de la sauvegarde du fichier PDF : {}", e))?;

    println!(
        "Le fichier PDF a été mis à jour : La police '{}' a remplacé '{}'.",
        font_name, police_a_remplacer
    );
    Ok(())
}
























// Fonction pour décoder une chaîne UTF-16BE hexadécimale
fn decode_pdf_string(encoded: &pdf_modification::Object) -> Option<String> {
    if let pdf_modification::Object::String(encoded_string, _) = encoded {
        // Vérifier si la chaîne commence par le BOM UTF-16BE (FEFF)
        if encoded_string.starts_with(&[0xFE, 0xFF]) {
            // Supprimer le BOM (premiers deux octets) et convertir en u16
            let utf16_data: Vec<u16> = encoded_string[2..]
                .chunks(2)
                .filter_map(|pair| {
                    if pair.len() == 2 {
                        Some(u16::from_be_bytes([pair[0], pair[1]]))
                    } else {
                        None
                    }
                })
                .collect();

            // Décoder la chaîne UTF-16
            return String::from_utf16(&utf16_data).ok();
        }
    }
    None
}






impl AppState {
    fn show_success_animation(&self, ctx: &egui::Context) {
        *self.show_success_icon.lock().unwrap() = true;

        let ctx_clone = ctx.clone();
        let show_success_icon = Arc::clone(&self.show_success_icon);

        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(3)); // Durée de 10 secondes
            ctx_clone.request_repaint();
            *show_success_icon.lock().unwrap() = false;
        });
    }
}


fn load_image_as_texture(ctx: &egui::Context, path: &str) -> Option<TextureHandle> {
    // Charger l'image avec la bibliothèque `image`
    if let Ok(image) = image::open(path) {
        let image = image.resize_exact(50, 50, image::imageops::FilterType::Lanczos3); // Redimensionner à 10x10
        let image = image.into_rgba8(); // Convertir en format RGBA
        let size = [image.width() as usize, image.height() as usize];
        let pixels = image.into_raw(); // Obtenir les données des pixels

        // Créer une texture pour `egui`
        Some(ctx.load_texture(
            "success_image",
            egui::ColorImage::from_rgba_unmultiplied(size, &pixels),
            egui::TextureOptions::LINEAR, // Utiliser TextureOptions ici
        ))
    } else {
        eprintln!("Erreur : Impossible de charger l'image {}", path);
        None
    }
}














// Interface graphique
impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Configurer le style personnalisé
        setup_custom_theme(ctx);
        // Clonage des états partagés pour une utilisation sécurisée
        let pdf_files = Arc::clone(&self.pdf_files);
        let root_folder = Arc::clone(&self.root_folder);
        let selected_format = Arc::clone(&self.selected_format);
        // Échelle appliquée lors de la prévisualisation (facteur d'agrandissement)
        let preview_scale_factor: f32 = 2.5;
        if self.success_texture.is_none() {
            self.success_texture = load_image_as_texture(ctx, "assets/image/success.png"); // Chemin vers l'image
        }
        


        // Panneau central pour gérer les fichiers et les actions
        egui::CentralPanel::default()
            .frame(egui::Frame::default().inner_margin(egui::Margin {
                left: 20.0,  // Espace à gauche
                right: 20.0, // Espace à droite
                top: 10.0,   // Espace en haut
                bottom: 10.0, // Espace en bas
            }))
            .show(ctx, |ui| {

                
                ui.heading("Sélecteur de PDF");
                
                // Bouton pour sélectionner le dossier racine
                if ui.button("Sélectionner le dossier racine").clicked() {
                    if let Some(folder) = NativeFileDialog::new()
                        .set_location("~/")
                        .show_open_single_dir()
                        .ok()
                        .flatten()
                    {
                        *root_folder.lock().unwrap() = Some(folder.clone());
                        let pdfs = find_pdfs(&folder);
                
                        // Créer une variable pour stocker les informations nécessaires pour les annotations
                        let pdf_files_info: Vec<_> = pdfs.clone();
                        
                        // Mettez à jour l'attribut pdf_files_info de l'état de l'application
                        *self.pdf_files_info.lock().unwrap() = pdf_files_info;
                
                        *pdf_files.lock().unwrap() = pdfs;
                    }
                }

                // Appliquer la couleur 
                ui.visuals_mut().override_text_color = Some(CUSTOM_TEXT_COLOR_0093D3);
                if let Some(root) = &*root_folder.lock().unwrap() {
                    ui.label(format!("Dossier racine : {}", root.display()));
                }

                // Liste des fichiers PDF
                ui.separator();
                // Appliquer la couleur 
                ui.visuals_mut().override_text_color = Some(CUSTOM_TEXT_COLOR_FAFAFA);
                ui.heading("Fichiers PDF trouvés :");
                
                
                // Appliquer la couleur
                ui.visuals_mut().override_text_color = Some(CUSTOM_TEXT_COLOR_0B84B8);
                let pdf_files_lock = pdf_files.lock().unwrap();

                if pdf_files_lock.len() > 4 {
                    // Si plus de 5 fichiers, afficher dans une zone défilable
                    egui::ScrollArea::vertical()
                        .max_height(150.0) // Limite la hauteur pour activer le défilement
                        .show(ui, |ui| {
                            for pdf in pdf_files_lock.iter() {
                                ui.label(pdf.path.display().to_string());
                            }
                        });
                } else {
                    // Si 5 fichiers ou moins, afficher en liste normale
                    for pdf in pdf_files_lock.iter() {
                        ui.label(pdf.path.display().to_string());
                    }
                }                           
                

                // Sélection du format de page
                ui.separator();
                // Appliquer la couleur 
                ui.visuals_mut().override_text_color = Some(CUSTOM_TEXT_COLOR_FAFAFA);
                ui.heading("Sélectionnez un format de page :");
                

                let mut format = *selected_format.lock().unwrap();
                ui.radio_value(&mut format, PDFFormat::None, "Aucun");
                ui.radio_value(&mut format, PDFFormat::A4, "A4");
                ui.radio_value(&mut format, PDFFormat::A5, "A5");
                ui.radio_value(&mut format, PDFFormat::USLetter, "US Letter");
                ui.radio_value(&mut format, PDFFormat::A4Landscape, "A4 (Paysage)");
                ui.radio_value(&mut format, PDFFormat::A5Landscape, "A5 (Paysage)");
                ui.radio_value(&mut format, PDFFormat::USLetterLandscape, "US Letter (Paysage)");
                ui.radio_value(&mut format, PDFFormat::Automatique, "Automatique");
                *selected_format.lock().unwrap() = format;
                ui.separator();
                ui.separator();







                // Ajoutez la palette de couleurs dans l'interface utilisateur
                ui.label("Sélectionnez une couleur pour l'annotation :");
                ui.add_space(5.0); // Ajouter un espace vertical de 5 pixels
                ui.horizontal(|ui| {
                    // Tableau des couleurs et noms pour éviter de répéter le code
                    let colors = [
                        (egui::Color32::BLACK, "Noir"),
                        (egui::Color32::RED, "Rouge"),
                        (egui::Color32::BLUE, "Bleu"),
                        (egui::Color32::from_rgb(255, 255, 0), "Jaune"),
                        (egui::Color32::GREEN, "Vert"),
                    ];

                    for (color, name) in colors.iter() {
                        // Dimensions du carré
                        let size = egui::Vec2::splat(24.0);

                        // Allouer la zone pour le carré et détecter les clics directement sur cette zone
                        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

                        // Vérifiez si la zone est cliquée
                        if response.clicked() {
                            self.selected_color = *color; // Met à jour la couleur sélectionnée pour la palette
                            self.selected_color_copie_1 = *color; // Copie pour la prévisualisation
                            self.selected_color_copie_2 = *color; // Copie pour la prévisualisation
                            println!("Couleur sélectionnée : {}", name);
                        }

                        let painter = ui.painter();

                        // Dessiner le carré coloré
                        painter.rect_filled(rect, egui::Rounding::none(), *color);

                        // Dessiner une fine bordure autour du carré pour le différencier du fond
                        painter.rect_stroke(
                            rect,
                            egui::Rounding::none(),
                            egui::Stroke::new(1.0, egui::Color32::GRAY), // Bordure grise fine
                        );

                        // Dessiner un contour blanc plus épais si le carré est sélectionné
                        let is_selected = color == &self.selected_color;
                        if is_selected {
                            painter.rect_stroke(
                                rect.expand(2.0), // Agrandit le contour légèrement pour plus de visibilité
                                egui::Rounding::none(),
                                egui::Stroke::new(2.0, egui::Color32::WHITE), // Contour blanc pour la sélection
                            );
                        }
                    }
                });










                // Appliquer la couleur au texte de l'étiquette
                ui.visuals_mut().override_text_color = Some(CUSTOM_TEXT_COLOR_FAFAFA);
                let mut custom_text = self.custom_text.lock().unwrap();
                let custom_text_str = &mut *custom_text; // Déverrouiller MutexGuard pour obtenir `String`
                ui.heading("Texte de l'annotation :");

                

                // Appliquer la couleur au texte de la zone d'édition
                ui.visuals_mut().override_text_color = Some(CUSTOM_TEXT_COLOR_FAFAFA);

                if ui.text_edit_singleline(custom_text_str).changed() {
                    // Si le texte a changé, mettez à jour custom_text_info
                    *self.custom_text_info.lock().unwrap() = custom_text_str.clone();
                }

                ui.separator();
                // Appliquer la couleur 
                ui.visuals_mut().override_text_color = Some(CUSTOM_TEXT_COLOR_FAFAFA);
                ui.heading("Options d'annotation :");
                // Appliquer la couleur 
                ui.visuals_mut().override_text_color = Some(CUSTOM_TEXT_COLOR_FAFAFA);

                let mut apply_to_all_pages = *self.apply_to_all_pages.lock().unwrap();
                ui.horizontal(|ui| {
                    ui.label("Appliquer sur :");
                    ui.radio_value(&mut apply_to_all_pages, true, "Toutes les pages");
                    ui.radio_value(&mut apply_to_all_pages, false, "Première page uniquement");
                });
                *self.apply_to_all_pages.lock().unwrap() = apply_to_all_pages;











                // Si un format est sélectionné, afficher la prévisualisation
                if format != PDFFormat::None {
                    if let Some(dimensions) = get_dimensions_by_format(format) {
                        let scaled_dimensions = scale_dimensions_by_half(dimensions);

                        
                        egui::SidePanel::right("right_panel").show(ctx, |ui| {
                            // Appliquer la couleur 
                            ui.visuals_mut().override_text_color = Some(CUSTOM_TEXT_COLOR_067DF4);
                            ui.heading("Prévisualisation de la page");
                            // Réinitialiser après utilisation
                            ui.visuals_mut().override_text_color = None;
                        
                            // Position de l'annotation
                            let mut annotation_pos = *self.annotation_position.lock().unwrap();
                            ui.horizontal(|ui| {
                                ui.label("Position de l'annotation :");
                                ui.add(egui::DragValue::new(&mut annotation_pos.0).speed(1.0).prefix("X: "));
                                ui.add(egui::DragValue::new(&mut annotation_pos.1).speed(1.0).prefix("Y: "));
                            });
                            *self.annotation_position.lock().unwrap() = annotation_pos;


                            let mut annotation_pos = *self.annotation_position.lock().unwrap();

                            // Activer le drag dans la zone de prévisualisation
                            let drag_response = ui.interact(
                                ui.available_rect_before_wrap(),
                                egui::Id::new("drag_annotation"),
                                egui::Sense::drag(),
                            );

                            if drag_response.dragged() {
                                let delta = drag_response.drag_delta();
                                annotation_pos.0 += delta.x as f64 / preview_scale_factor as f64;
                                annotation_pos.1 -= delta.y as f64 / preview_scale_factor as f64;
                            }

                            // Sauvegarder la position mise à jour
                            *self.annotation_position.lock().unwrap() = annotation_pos;

                        
                            // Définir la taille disponible pour la prévisualisation
                            let available_size = egui::Vec2::new(
                                scaled_dimensions.0.0 as f32 * preview_scale_factor,
                                scaled_dimensions.1.0 as f32 * preview_scale_factor,
                            );
                        
                            // Allouer la zone de prévisualisation
                            let (response, painter) = ui.allocate_painter(available_size, egui::Sense::hover());
                            let rect = response.rect;
                        
                            // Étape 1 : Dessiner un arrière-plan (optionnel)
                            painter.rect_filled(
                                rect,
                                egui::Rounding::none(),
                                egui::Color32::from_rgb(89, 89, 89), // Un gris léger comme arrière-plan
                            );
                        
                            // Étape 2 : Dessiner le rectangle représentant la page (avec la mise à l'échelle)
                            painter.rect_filled(
                                rect,
                                egui::Rounding::none(),
                                egui::Color32::WHITE, // La page est représentée en blanc
                            );

                            
                            // Étape 3 : Dessiner le cadre autour du rectangle
                            painter.rect_stroke(
                                rect,
                                egui::Rounding::none(),
                                egui::Stroke::new(2.0, egui::Color32::BLUE), // Contour de la page
                            );
                        
                            // Étape 4 : Calculer et dessiner les annotations sur la page
                            let annotation_pos_scaled = (
                                annotation_pos.0 * preview_scale_factor as f64,
                                annotation_pos.1 * preview_scale_factor as f64,
                            );
                        
                            // Calculer la position du texte dans la prévisualisation (inverser Y pour que l'origine soit en bas à gauche)
                            let text_pos = rect.min + egui::vec2(
                                annotation_pos_scaled.0 as f32, // Convertir `f64` en `f32`
                                available_size.y - annotation_pos_scaled.1 as f32, // Convertir `f64` en `f32` avant la soustraction
                            );
                            
                            
                        
                            // Déterminer la couleur de l'annotation (par exemple rouge si elle dépasse les limites)
                            let annotation_color = if annotation_pos.0 < 0.0 || annotation_pos.1 > 145.0 {
                                egui::Color32::RED
                            } else {
                                self.selected_color_copie_1 // Couleur sélectionnée dnas la palette de couleur
                            };
                        
                            // Dessiner le texte d'annotation
                            painter.text(
                                text_pos,
                                egui::Align2::LEFT_BOTTOM, // Modifier Align2 pour que le texte soit aligné en bas à gauche
                                &custom_text,
                                egui::FontId::new(9.0, egui::FontFamily::Proportional), // Définir une taille de police pour ajuster a la taille de la prévisualisation
                                annotation_color,
                            );
                            // Réinitialiser après utilisation
                            ui.visuals_mut().override_text_color = None;
                        });
                        
                        
                    }
                }



                // Pup UP
                if *self.show_no_root_popup.lock().unwrap() {
                    let screen_center = ctx.screen_rect().center(); // Obtenir le centre de l'écran
                    
                    // Définir les styles personnalisés pour le popup
                    let frame = egui::Frame::none()
                        .fill(egui::Color32::from_rgb(60, 0, 0)) // Gris foncé mélangé à du rouge
                        .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(120, 0, 0))) // Bordure rouge foncé
                        .rounding(egui::Rounding::same(10.0)); // Coins arrondis
                
                    egui::Window::new("Dossier racine non sélectionné")
                        .frame(frame) // Appliquer le style personnalisé
                        .resizable(false)
                        .collapsible(false)
                        .fixed_pos(screen_center - egui::vec2(100.0, 50.0)) // Positionner le popup au centre
                        .show(ctx, |ui| {
                            ui.vertical(|ui| {
                                ui.add_space(10.0); // Ajouter un espace avant le texte
                                ui.label(egui::RichText::new("Veuillez sélectionner un dossier racine avant de continuer.")
                                    .color(egui::Color32::RED) // Texte en rouge
                                    .size(16.0)); // Taille du texte
                
                                ui.add_space(10.0); // Ajouter un espace après le texte
                
                                if ui.button("OK").clicked() {
                                    *self.show_no_root_popup.lock().unwrap() = false; // Fermer la popup
                                }
                            });
                        });
                }
                
                
                
                if *self.show_no_pdf_popup.lock().unwrap() {
                    let screen_center = ctx.screen_rect().center(); // Obtenir le centre de l'écran
                
                    // Style personnalisé pour le popup
                    let frame = egui::Frame::none()
                        .fill(egui::Color32::from_rgb(45, 45, 45)) // Fond gris foncé
                        .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(200, 0, 0))) // Bordure rouge vif
                        .rounding(egui::Rounding::same(10.0)); // Coins arrondis
                
                    egui::Window::new("Aucun fichier PDF")
                        .frame(frame) // Appliquer le style
                        .resizable(false)
                        .collapsible(false)
                        .fixed_pos(screen_center - egui::vec2(150.0, 50.0)) // Ajustez selon la taille du popup
                        .show(ctx, |ui| {
                            ui.vertical(|ui| {
                                ui.add_space(10.0); // Ajouter de l'espace avant le texte
                                ui.label(egui::RichText::new("Aucun fichier PDF trouvé dans la liste. Veuillez en ajouter avant de continuer.")
                                    .color(egui::Color32::RED) // Texte en rouge
                                    .size(16.0)); // Taille du texte
                
                                ui.add_space(10.0); // Ajouter de l'espace après le texte
                
                                if ui.button("OK").clicked() {
                                    *self.show_no_pdf_popup.lock().unwrap() = false; // Fermer la popup
                                }
                            });
                        });
                }
                
                
                if *self.show_pdf_open.lock().unwrap() {
                    let screen_center = ctx.screen_rect().center(); // Obtenir le centre de l'écran
                
                    // Style personnalisé pour le popup
                    let frame = egui::Frame::none()
                        .fill(egui::Color32::from_rgb(55, 0, 0)) // Fond rouge foncé
                        .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 69, 0))) // Bordure orange vif
                        .rounding(egui::Rounding::same(10.0)); // Coins arrondis
                
                    egui::Window::new("Erreur : Fichier utilisé")
                        .frame(frame) // Appliquer le style
                        .resizable(false)
                        .collapsible(false)
                        .fixed_pos(screen_center - egui::vec2(200.0, 75.0)) // Ajustez selon la taille du popup
                        .show(ctx, |ui| {
                            ui.vertical(|ui| {
                                ui.add_space(10.0); // Ajouter de l'espace avant le texte
                                ui.label(egui::RichText::new("Un ou plusieurs fichiers sont actuellement ouverts ou utilisés par un autre processus. Veuillez les fermer avant de continuer.")
                                    .color(egui::Color32::RED) // Texte en rouge vif
                                    .size(16.0)); // Taille du texte
                
                                ui.add_space(10.0); // Ajouter de l'espace après le texte
                
                                if ui.button("OK").clicked() {
                                    *self.show_pdf_open.lock().unwrap() = false; // Fermer la popup
                                }
                            });
                        });
                }


                

                if *self.show_annotation_removal_popup.lock().unwrap() {
                    let screen_center = ctx.screen_rect().center(); // Obtenir le centre de l'écran
                
                    // Style personnalisé pour la popup
                    let frame = egui::Frame::none()
                        .fill(egui::Color32::from_rgb(45, 45, 45)) // Fond gris foncé
                        .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 200, 0))) // Bordure verte
                        .rounding(egui::Rounding::same(10.0)); // Coins arrondis
                
                    egui::Window::new("Suppression des annotations terminée")
                        .frame(frame) // Appliquer le style
                        .resizable(false)
                        .collapsible(false)
                        .fixed_pos(screen_center - egui::vec2(150.0, 50.0)) // Ajuster selon la taille de la popup
                        .show(ctx, |ui| {
                            ui.vertical(|ui| {
                                ui.add_space(10.0); // Ajouter un espace avant le texte
                                let count = *self.annotation_removal_count.lock().unwrap();
                                ui.label(egui::RichText::new(format!("Annotations supprimées dans {} fichier(s).", count))
                                    .color(egui::Color32::GREEN) // Texte en vert
                                    .size(16.0)); // Taille du texte
                
                                ui.add_space(10.0); // Ajouter un espace après le texte
                
                                if ui.button("Fermer").clicked() {
                                    *self.show_annotation_removal_popup.lock().unwrap() = false; // Fermer la popup
                                }
                            });
                        });
                }

                
                
                
                // Condition pour afficher la fenêtre
                if *self.show_success_icon.lock().unwrap() {
                    if let Some(texture) = &self.success_texture {
                        egui::Area::new("success_image")
                            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO) // Positionner au centre
                            .show(ctx, |ui| {
                                // Afficher l'image avec sa taille d'origine
                                ui.image(texture, texture.size_vec2());
                            });
                    } else {
                        eprintln!("Erreur : l'image de succès n'a pas pu être chargée.");
                    }
                }
                
                
                
                
                // -------------------                

                




                // Bouton pour créer un fichier PDF avec le nom défini par l'utilisateur
                // Appliquer la couleur 
                ui.visuals_mut().override_text_color = Some(CUSTOM_TEXT_COLOR_FAFAFA);
                if ui.button("Nouveau_Fichier PDF").clicked() {
                    if let Some(root) = &*root_folder.lock().unwrap() {
                        if format != PDFFormat::None {
                            if let Some(dimensions) = get_dimensions_by_format(format) {
                                // Afficher la boîte de dialogue de sauvegarde pour choisir le nom du fichier
                                if let Some(mut save_path) = RfdFileDialog::new()
                                    .set_directory(root)
                                    .set_file_name("nouveau_fichier.pdf")
                                    .add_filter("PDF Files", &["pdf"]) // Spécifier le filtre pour les fichiers PDF
                                    .save_file()
                                {
                                    // Ajouter l'extension .pdf si l'utilisateur ne l'a pas spécifiée
                                    if save_path.extension().map_or(true, |ext| ext != "pdf") {
                                        save_path.set_extension("pdf");
                                    }

                                    // Essayer de créer ou d'ouvrir le fichier pour vérifier s'il est déjà utilisé
                                    match File::create(&save_path) {
                                        Ok(output_file) => {
                                            let annotation_pos = *self.annotation_position.lock().unwrap();
                                            let scaled_dimensions = scale_dimensions_by_half(dimensions);
                                            let original_dimensions = restore_original_dimensions(scaled_dimensions);

                                            // Calculer les facteurs d'échelle pour convertir les dimensions de la prévisualisation en dimensions réelles
                                            let scale_factor_x = original_dimensions.0.0 / scaled_dimensions.0.0; // Facteur d'échelle en largeur
                                            let scale_factor_y = original_dimensions.1.0 / scaled_dimensions.1.0; // Facteur d'échelle en hauteur

                                            // Créer un nouveau document PDF avec printpdf
                                            let (doc, page1, layer1) = pdf_creation::PdfDocument::new(
                                                "Test PDF",
                                                original_dimensions.0,
                                                original_dimensions.1,
                                                "Layer 1",
                                            );

                                            

                                            let current_layer = doc.get_page(page1).get_layer(layer1);

                                            

                                            // Adapter la position de l'annotation en fonction de l'échelle de prévisualisation
                                            let annotation_pos_scaled = (
                                                annotation_pos.0 as f64 * scale_factor_x,
                                                annotation_pos.1 as f64 * scale_factor_y,
                                            );

                                            let custom_text_str = &*custom_text; // Obtenir une référence de `String` en `str`


                                            // Convertir la couleur sélectionnée en RGB pour printpdf
                                            let palette_color = self.selected_color_copie_2;
                                            let color_rgb = (
                                                palette_color.r() as f64 / 255.0,
                                                palette_color.g() as f64 / 255.0,
                                                palette_color.b() as f64 / 255.0,
                                            );

                                            // Définir la couleur de remplissage pour le texte
                                            current_layer.set_fill_color(printpdf::Color::Rgb(printpdf::Rgb::new(
                                                color_rgb.0,
                                                color_rgb.1,
                                                color_rgb.2,
                                                None,
                                            )));


                                            // Ajouter le texte avec la couleur sélectionnée
                                            current_layer.use_text(
                                                custom_text_str,
                                                18.0,
                                                Mm(annotation_pos_scaled.0),
                                                Mm(annotation_pos_scaled.1),
                                                &doc.add_builtin_font(BuiltinFont::HelveticaBold).unwrap(),
                                            );

                                            // Sauvegarder le document avec le chemin spécifié par l'utilisateur
                                            let mut buf_writer = BufWriter::new(output_file);
                                            if let Err(e) = doc.save(&mut buf_writer) {
                                                eprintln!("Erreur lors de la sauvegarde du fichier PDF : {}", e);
                                            } else {
                                                println!("Fichier PDF créé : {}", save_path.display());
                                            }
                                        }
                                        Err(_) => {
                                            // Afficher une boîte de dialogue d'erreur si le fichier est déjà ouvert/utilisé
                                            MessageDialog::new()
                                                .set_type(MessageType::Error)
                                                .set_title("Erreur")
                                                .set_text("Le fichier est déjà ouvert ou en cours d'utilisation. Veuillez le fermer et réessayer.")
                                                .show_alert()
                                                .unwrap();
                                        }
                                    }

                                    // Appeler l'animation de succès
                                    println!("Fin de l'ajout des annotations aux fichiers PDF.");
                                    *self.show_success_icon.lock().unwrap() = true;
                                    self.show_success_animation(ctx); // Appeler l'animation de succès
                                }
                            }
                        }
                    } else {
                        // Aucun dossier racine sélectionné, afficher une popup
                        *self.show_no_root_popup.lock().unwrap() = true;
                    } 
                }












// Les anotation sont en U16 With BOM







                // Bouton pour déclencher l'ajout récursif d'annotations
                // Appliquer la couleur 
                ui.visuals_mut().override_text_color = Some(CUSTOM_TEXT_COLOR_FAFAFA);
                if ui.button("Annotation Récursive").clicked() {
                    println!("Début de l'annotation récursive");

                    // Vérifier si un fichier PDF est en cours d'utilisation
                    let pdf_files_info = self.pdf_files_info.lock().unwrap().clone();
                    for pdf in &pdf_files_info {
                        match std::fs::OpenOptions::new().write(true).open(&pdf.path) {
                            Ok(_) => {
                                println!("Le fichier {} est accessible.", pdf.path.display());
                            }
                            Err(_) => {
                                // Afficher un popup pour informer que le fichier est déjà utilisé
                                *self.show_pdf_open.lock().unwrap() = true;
                    
                                println!(
                                    "Popup : Le fichier '{}' est actuellement ouvert ou utilisé par un autre processus.",
                                    pdf.path.display()
                                );
                    
                                return; // Arrêter la logique ici si un fichier est en cours d'utilisation
                            }
                        }
                    }
                    

                    // Vérifie si un dossier racine est sélectionné
                    let root_selected = self.root_folder.lock().unwrap().is_some();
                    // Vérifiez si le format sélectionné est "Automatique"
                    let is_automatic_mode = *self.selected_format.lock().unwrap() == PDFFormat::Automatique;

                    // Vérifie si des fichiers PDF sont présents
                    let pdf_files_info = self.pdf_files_info.lock().unwrap().clone();
                    let pdfs_available = !pdf_files_info.is_empty();

                    if !root_selected {
                        // Si aucun dossier racine n'est sélectionné
                        *self.show_no_root_popup.lock().unwrap() = true;
                        println!("Popup : Aucun dossier racine sélectionné.");
                    } else if !pdfs_available {
                        // Si aucun fichier PDF n'est trouvé dans le dossier sélectionné
                        *self.show_no_pdf_popup.lock().unwrap() = true;
                        println!("Popup : Aucun fichier PDF trouvé dans le dossier sélectionné.");
                    } else {
                        // Verifier ci le mode automatique est acctif !
                        if is_automatic_mode {
                            println!("Mode automatique activé. Prévisualisation basée sur le format A4.");
                        
                            // Déverrouiller pour accéder à la position de l'annotation
                            let annotation_position = {
                                let annotation_position_lock = self.annotation_position.lock().unwrap();
                                *annotation_position_lock
                            };
                        
                            // Échelle effective de la prévisualisation (réduction à 50 %, puis agrandissement de 2.5)
                            let scaled_dimensions = scale_dimensions_by_half((Mm(210.0), Mm(297.0)));
                            let original_dimensions = restore_original_dimensions(scaled_dimensions);
                        
                            println!(
                                "Dimensions A4 (prévisualisation réduite) : Largeur = {:.2} mm, Hauteur = {:.2} mm",
                                scaled_dimensions.0 .0, scaled_dimensions.1 .0
                            );
                        
                            // Calculer les facteurs d'échelle pour ramener les dimensions à 1:1
                            let scale_factor_x = original_dimensions.0 .0 / scaled_dimensions.0 .0;
                            let scale_factor_y = original_dimensions.1 .0 / scaled_dimensions.1 .0;
                        
                            // Adapter la position de l'annotation à une échelle 1:1 pour A4
                            let annotation_position_real = (
                                annotation_position.0 * scale_factor_x,
                                annotation_position.1 * scale_factor_y,
                            );
                        
                            println!(
                                "Position réelle dans A4 (échelle 1:1) : X = {:.2}, Y = {:.2}",
                                annotation_position_real.0, annotation_position_real.1
                            );
                        
                            // Itérer sur chaque fichier PDF
                            for (index, pdf) in pdf_files_info.iter().enumerate() {
                                println!("\n=== Traitement du fichier PDF {} : {} ===", index + 1, pdf.path.display());
                        
                                // Détecter les dimensions du PDF
                                match get_page_dimensions(&pdf.path) {
                                    Some((format, (width_mm, height_mm))) => {
                                        println!(
                                            "Dimensions trouvées : Format = {}, Largeur = {:.2} mm, Hauteur = {:.2} mm",
                                            format, width_mm, height_mm
                                        );
                        
                                        // Vérifier si le format est A4
                                        let annotation_pos_points = if format == "A4" {
                                            println!("Le format est déjà en A4. Pas de conversion nécessaire.");
                                            (
                                                annotation_position_real.0 * 2.83465, // Conversion en points
                                                annotation_position_real.1 * 2.83465, // Conversion en points
                                            )
                                        } else {
                                            // Ajuster la position pour les formats non A4
                                            let adjusted_position = adjust_position(
                                                (original_dimensions.0 .0, original_dimensions.1 .0), // Dimensions de prévisualisation
                                                (210.0, 297.0),                                       // Dimensions A4 portrait (origine)
                                                (width_mm, height_mm),                                // Dimensions réelles du PDF cible
                                                annotation_position_real                              // Position actuelle de l'annotation
                                            );
                                            
                                            
                        
                                            println!(
                                                "Position ajustée pour le format cible : X = {:.2} mm, Y = {:.2} mm",
                                                adjusted_position.0, adjusted_position.1
                                            );
                        
                                            (
                                                adjusted_position.0 * 2.83465, // Conversion en points
                                                adjusted_position.1 * 2.83465, // Conversion en points
                                            )
                                        };
                        
                                        println!(
                                            "Position finale de l'annotation (points) : X = {:.2}, Y = {:.2}",
                                            annotation_pos_points.0, annotation_pos_points.1
                                        );
                        
                                        // Charger et modifier le PDF
                                        match pdf_modification::Document::load(&pdf.path) {
                                            Ok(mut doc) => {
                                                println!("Chargement réussi du fichier PDF : {}", pdf.path.display());
                        
                                                let pages = doc.get_pages();
                                                println!("Nombre de pages trouvées : {}", pages.len());
                        
                                                for (page_number, page_id) in pages {
                                                    println!("Traitement de la page {}", page_number);
                                                    // Vérifier si on doit annoter uniquement la première page
                                                    if !apply_to_all_pages && page_number > 1 {
                                                        break; // Arrêter si on ne traite que la première page
                                                    }
                        
                                                    // Calculer les dimensions de la boîte de texte
                                                    let (rect_width_pts, rect_height_pts) =
                                                        match calculate_textbox_dimensions(&custom_text, 18.0) {
                                                            Some((width_mm, height_mm)) => {
                                                                let mm_to_points = 2.83465; // Conversion de millimètres en points
                                                                (width_mm * mm_to_points, height_mm * mm_to_points)
                                                            }
                                                            None => {
                                                                eprintln!("Erreur lors du calcul des dimensions de la boîte de texte. Utilisation de valeurs par défaut.");
                                                                (100.0, 20.0) // Valeurs par défaut (en points)
                                                            }
                                                        };
                        
                                                    // Définir les coordonnées du rectangle
                                                    let rect = vec![
                                                        annotation_pos_points.0.into(),
                                                        annotation_pos_points.1.into(),
                                                        (annotation_pos_points.0 + rect_width_pts).into(),
                                                        (annotation_pos_points.1 - rect_height_pts).into(),
                                                    ];
                        
                                                    println!(
                                                        "Rectangle d'annotation : [X_min: {:.2}, Y_min: {:.2}, X_max: {:.2}, Y_max: {:.2}]",
                                                        annotation_pos_points.0,
                                                        annotation_pos_points.1,
                                                        annotation_pos_points.0 + rect_width_pts,
                                                        annotation_pos_points.1 - rect_height_pts
                                                    );
                        
                                                    // Encodage du texte en UTF-16BE avec BOM
                                                    let mut encoded_text = vec![0xFE, 0xFF]; // UTF-16BE BOM
                                                    encoded_text.extend(
                                                        custom_text.encode_utf16().flat_map(|u| u.to_be_bytes())
                                                    );

                                                    // Convertionne couleur de la palette en RGB
                                                    let palette_color = self.selected_color_copie_2;
                                                    let color_rgb = format!(
                                                        "{:.2} {:.2} {:.2} rg",
                                                        palette_color.r() as f32 / 255.0, // Rouge
                                                        palette_color.g() as f32 / 255.0, // Vert
                                                        palette_color.b() as f32 / 255.0  // Bleu
                                                    );
                        
                                                    // Créer le dictionnaire d'annotation
                                                    let annotation = pdf_modification::dictionary! {
                                                        "Type" => "Annot",
                                                        "Subtype" => "FreeText",
                                                        "Rect" => pdf_modification::Object::Array(rect),
                                                        "Contents" => pdf_modification::Object::String(encoded_text.into(), StringFormat::Hexadecimal),
                                                        "DA" => pdf_modification::Object::string_literal(format!("/F1 18 Tf {}", color_rgb)),
                                                        "F" => pdf_modification::Object::Integer(4),
                                                    };
                        
                                                    let annotation_id = doc.add_object(annotation);
                        
                                                    // Ajouter l'annotation au dictionnaire de la page
                                                    if let Ok(page_dict) = doc.get_object_mut(page_id) {
                                                        if let pdf_modification::Object::Dictionary(ref mut dict) = page_dict {
                                                            let mut annotations = match dict.get(b"Annots") {
                                                                Ok(pdf_modification::Object::Array(ref annots)) => annots.clone(),
                                                                _ => vec![],
                                                            };
                                                            annotations.push(pdf_modification::Object::Reference(annotation_id));
                                                            dict.set("Annots", pdf_modification::Object::Array(annotations));
                                                        }
                                                    }
                        
                                                    
                                                }
                        
                                                let temp_path = pdf.path.with_extension("temp.pdf");
                                                if let Err(e) = doc.save(&temp_path) {
                                                    eprintln!("Erreur lors de la sauvegarde du fichier temporaire : {}", e);
                                                    continue; // Passer au fichier suivant
                                                }
                                                if let Err(e) = std::fs::rename(temp_path, &pdf.path) {
                                                    eprintln!("Erreur lors du remplacement du fichier PDF : {}", e);
                                                } else {
                                                    println!("Annotations ajoutées avec succès au fichier : {}", pdf.path.display());
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!(
                                                    "Erreur lors du chargement du fichier PDF {} : {}",
                                                    pdf.path.display(),
                                                    e
                                                );
                                            }
                                        }
                                    }
                                    None => {
                                        println!(
                                            "Erreur : Impossible de détecter les dimensions du fichier {}",
                                            pdf.path.display()
                                        );
                                    }
                                }
                            }
                            // Appeler l'animation de succès
                            println!("Fin de l'ajout des annotations aux fichiers PDF.");
                            *self.show_success_icon.lock().unwrap() = true;
                            self.show_success_animation(ctx); // Appeler l'animation de succès
                        }
                                                                                                                                       
                         else {
                            // Si toutes les vérifications passent, exécuter la logique pour ajouter les annotations
                            println!("Tous les fichiers et dossiers requis sont disponibles. Ajout des annotations...");
                            

                            // Lire le dossier racine
                            if let Some(_root) = {
                                let root_lock = self.root_folder.lock().unwrap();
                                root_lock.clone()
                            } {
                                println!("1ère condition de l'annotation récursive remplie");

                                // Utiliser `pdf_files_info` depuis l'état de l'application
                                let pdf_files_info = {
                                    let pdf_files_info_lock = self.pdf_files_info.lock().unwrap();
                                    pdf_files_info_lock.clone() // Clone la liste des fichiers depuis le verrou, puis libère le verrou immédiatement
                                };
                                println!("Liste des fichiers PDF récupérée");

                                if !pdf_files_info.is_empty() {
                                    println!("2ème condition de l'annotation récursive remplie");

                                    // *** Étape : Ajout des Annotations aux Fichiers PDF ***
                                    println!("Début de l'ajout des annotations aux fichiers PDF...");
                                    let annotation_position = {
                                        let annotation_position_lock = self.annotation_position.lock().unwrap();
                                        *annotation_position_lock
                                    };
                                    println!(
                                        "Position de l'annotation récupérée : ({}, {})",
                                        annotation_position.0, annotation_position.1
                                    );

                                    // Utiliser `custom_text_info` ici pour éviter de devoir verrouiller `custom_text` plusieurs fois
                                    let custom_text = {
                                        let custom_text_info_lock = self.custom_text_info.lock().unwrap();
                                        custom_text_info_lock.clone()
                                    };

                                    // Obtenir le format sélectionné
                                    let format = *self.selected_format.lock().unwrap();

                                    if let Some(dimensions) = get_dimensions_by_format(format) {
                                        // Réduction des dimensions pour la prévisualisation
                                        let scaled_dimensions = scale_dimensions_by_half(dimensions);
                                        let original_dimensions = restore_original_dimensions(scaled_dimensions);

                                        // Calculer les facteurs d'échelle pour convertir les dimensions de la prévisualisation en dimensions réelles
                                        let scale_factor_x = original_dimensions.0.0 / scaled_dimensions.0.0; // Facteur d'échelle en largeur
                                        let scale_factor_y = original_dimensions.1.0 / scaled_dimensions.1.0; // Facteur d'échelle en hauteur

                                        // Adapter la position de l'annotation en fonction de l'échelle de prévisualisation
                                        let annotation_pos_scaled = (
                                            annotation_position.0 * scale_factor_x,
                                            annotation_position.1 * scale_factor_y,
                                        );

                                        // Conversion des dimensions et positions en points (1 mm = 2.83465 points)
                                        let mm_to_points = 2.83465;
                                        let annotation_pos_points = (
                                            annotation_pos_scaled.0 * mm_to_points,
                                            annotation_pos_scaled.1 * mm_to_points,
                                        );


                                        // Ajuster la position pour correspondre à l'origine du PDF avec inversion de l'axe Y
                                        let annotation_pos_adjusted = (
                                            annotation_pos_points.0,
                                            annotation_pos_points.1,
                                        );

                                        println!(
                                            "Position finale de l'annotation : X = {:.2}, Y = {:.2}",
                                            annotation_pos_adjusted.0, annotation_pos_adjusted.1
                                        );

                                        for (_index, pdf) in pdf_files_info.iter().enumerate() {
                                            println!("Ajout des annotations au fichier : {}", pdf.path.display());
                                            let file_path = &pdf.path;

                                            // Charger le document PDF existant avec `lopdf`
                                            match pdf_modification::Document::load(file_path) {
                                                Ok(mut doc) => {
                                                    println!("Chargement réussi du fichier PDF : {}", file_path.display());

                                                    // Obtenir la liste des pages du document
                                                    let pages = doc.get_pages();
                                                    println!("Nombre de pages trouvées : {}", pages.len());

                                                    for (page_number, page_id) in pages {
                                                        println!("Traitement de la page {}", page_number);

                                                        // Calculer les dimensions de la boîte de texte dynamiquement
                                                        let (rect_width_pts, rect_height_pts) = match calculate_textbox_dimensions(&custom_text, 18.0) {
                                                            Some((width_mm, height_mm)) => {
                                                                let mm_to_points = 2.83465; // Conversion de millimètres en points
                                                                (width_mm * mm_to_points, height_mm * mm_to_points)
                                                            }
                                                            None => {
                                                                eprintln!("Erreur lors du calcul des dimensions de la boîte de texte. Utilisation de valeurs par défaut.");
                                                                (100.0, 20.0) // Valeurs par défaut (en points)
                                                            }
                                                        };

                                                        // Définir les coordonnées du rectangle
                                                        let rect = vec![
                                                            annotation_pos_adjusted.0.into(),
                                                            annotation_pos_adjusted.1.into(),
                                                            (annotation_pos_adjusted.0 + rect_width_pts).into(),
                                                            (annotation_pos_adjusted.1 - rect_height_pts).into(),
                                                        ];

                                                        println!(
                                                            "Rectangle d'annotation : [X_min: {:.2}, Y_min: {:.2}, X_max: {:.2}, Y_max: {:.2}]",
                                                            annotation_pos_adjusted.0,
                                                            annotation_pos_adjusted.1,
                                                            annotation_pos_adjusted.0 + rect_width_pts,
                                                            annotation_pos_adjusted.1 - rect_height_pts
                                                        );

                                                        // Encodage du texte de l'annotation en UTF-16BE avec BOM
                                                        let mut encoded_text = vec![0xFE, 0xFF]; // UTF-16BE BOM
                                                        encoded_text.extend(
                                                            custom_text.encode_utf16().flat_map(|u| u.to_be_bytes())
                                                        );

                                                        // Créer le dictionnaire d'annotation
                                                        let annotation = pdf_modification::dictionary! {
                                                            "Type" => "Annot",
                                                            "Subtype" => "FreeText",
                                                            "Rect" => pdf_modification::Object::Array(rect),
                                                            "Contents" => pdf_modification::Object::String(encoded_text.into(), StringFormat::Hexadecimal),
                                                            "DA" => pdf_modification::Object::string_literal("/F1 18 Tf 0 0 0 rg"),
                                                            "F" => pdf_modification::Object::Integer(4),
                                                        };

                                                        let annotation_id = doc.add_object(annotation);

                                                        // Ajouter l'annotation au dictionnaire de la page
                                                        if let Ok(page_dict) = doc.get_object_mut(page_id) {
                                                            if let pdf_modification::Object::Dictionary(ref mut dict) = page_dict {
                                                                let mut annotations = match dict.get(b"Annots") {
                                                                    Ok(pdf_modification::Object::Array(ref annots)) => annots.clone(),
                                                                    _ => vec![],
                                                                };
                                                                annotations.push(pdf_modification::Object::Reference(annotation_id));
                                                                dict.set("Annots", pdf_modification::Object::Array(annotations));
                                                            }
                                                        }


                                                        if !apply_to_all_pages && page_number > 1 {
                                                            break; // Sortir de la boucle si on ne doit traiter que la première page
                                                        }
                                                    }

                                                    let temp_path = file_path.with_extension("temp.pdf");
                                                    doc.save(&temp_path)
                                                        .expect("Erreur lors de la sauvegarde du fichier PDF.");
                                                    std::fs::rename(temp_path, file_path)
                                                        .expect("Erreur lors du remplacement du fichier PDF.");
                                                }
                                                Err(e) => {
                                                    eprintln!(
                                                        "Erreur lors du chargement du fichier PDF {} : {}",
                                                        file_path.display(),
                                                        e
                                                    );
                                                }
                                            }
                                        }
                                        println!("Fin de l'ajout des annotations aux fichiers PDF.");
                                    } else {
                                        eprintln!("Erreur : Format de page non sélectionné.");
                                    }
                                }
                            }
                            // Appeler l'animation de succès
                            println!("Fin de l'ajout des annotations aux fichiers PDF.");
                            *self.show_success_icon.lock().unwrap() = true;
                            self.show_success_animation(ctx); // Appeler l'animation de succès
                        }
                    }
                }














                // Bouton pour supprimer les annotations ayant le même texte
                ui.visuals_mut().override_text_color = Some(CUSTOM_TEXT_COLOR_FAFAFA);
                if ui.button("Supprimer les annotations").clicked() {
                    println!("Début de la suppression des annotations...");

                    // Étape 1 : Vérifier si un dossier racine est sélectionné
                    let root_selected = {
                        let root_lock = self.root_folder.lock().unwrap();
                        root_lock.is_some()
                    };
                    if !root_selected {
                        *self.show_no_root_popup.lock().unwrap() = true;
                        println!("Popup : Aucun dossier racine sélectionné.");
                        return;
                    }

                    // Étape 2 : Vérifier si des fichiers PDF sont trouvés
                    let pdf_files_info = {
                        let pdf_files_lock = self.pdf_files_info.lock().unwrap();
                        pdf_files_lock.clone()
                    };
                    if pdf_files_info.is_empty() {
                        *self.show_no_pdf_popup.lock().unwrap() = true;
                        println!("Popup : Aucun fichier PDF trouvé pour la suppression des annotations.");
                        return;
                    }

                    // Étape 3 : Vérifier si un fichier PDF est déjà ouvert
                    for pdf in &pdf_files_info {
                        if let Err(_) = std::fs::OpenOptions::new().write(true).open(&pdf.path) {
                            println!(
                                "Le fichier {} est actuellement ouvert ou utilisé par un autre processus.",
                                pdf.path.display()
                            );
                            *self.show_pdf_open.lock().unwrap() = true;
                            return; // Arrêter ici si un fichier est ouvert
                        }
                    }

                    // Étape 4 : Récupérer le texte cible à rechercher pour éviter de bloquer un verrou prolongé
                    let target_text = self.custom_text_info.lock().unwrap().clone();
                    println!("Texte cible à rechercher : {}", target_text);

                    // Étape 5 : Vérifier si toutes les pages doivent être analysées ou seulement la première
                    let apply_to_all_pages = *self.apply_to_all_pages.lock().unwrap();

                    // Étape 6 : Parcourir les fichiers PDF pour supprimer les annotations
                    let mut files_with_removed_annotations = 0; // Compteur pour les fichiers modifiés
                    for pdf in pdf_files_info {
                        println!("Traitement du fichier : {}", pdf.path.display());

                        // Charger le document PDF
                        let mut doc = match pdf_modification::Document::load(&pdf.path) {
                            Ok(doc) => doc,
                            Err(e) => {
                                eprintln!(
                                    "Erreur lors du chargement initial du fichier {} : {}",
                                    pdf.path.display(),
                                    e
                                );

                                // Si le chargement échoue, tenter une réparation avec `repair_and_reload_pdf`
                                match repair_and_reload_pdf(&pdf.path) {
                                    Ok(repaired_doc) => repaired_doc,
                                    Err(repair_err) => {
                                        eprintln!(
                                            "Échec de la réparation pour le fichier {} : {}",
                                            pdf.path.display(),
                                            repair_err
                                        );
                                        continue; // Passer au fichier suivant si la réparation échoue
                                    }
                                }
                            }
                        };

                        println!("Chargement réussi du fichier PDF.");

                        let mut annotations_removed = false;

                        // Parcourir les pages du document
                        for (page_number, page_id) in doc.get_pages() {
                            println!("Traitement de la page {}", page_number);

                            // Si `apply_to_all_pages` est `false` et que ce n'est pas la première page, sortir de la boucle
                            if !apply_to_all_pages && page_number > 1 {
                                break;
                            }

                            // Récupérer les annotations
                            if let Ok(pdf_modification::Object::Dictionary(page_dict)) = doc.get_object(page_id) {
                                if let Ok(annots) = page_dict.get(b"Annots") {
                                    if let pdf_modification::Object::Array(annot_refs) = annots {
                                        let mut updated_annots = annot_refs.clone();

                                        // Parcourir et vérifier chaque annotation
                                        for annot_ref in annot_refs {
                                            if let Ok(annot_obj) = doc.get_object(annot_ref.as_reference().unwrap()) {
                                                if let pdf_modification::Object::Dictionary(annot_dict) = annot_obj {
                                                    if let Ok(contents) = annot_dict.get(b"Contents") {
                                                        if let Some(decoded_text) = decode_pdf_string(contents) {
                                                            if decoded_text == target_text {
                                                                println!("Annotation supprimée : {}", decoded_text);
                                                                updated_annots.retain(|r| r != annot_ref); // Supprimer l'annotation
                                                                annotations_removed = true;
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        // Mettre à jour les annotations sur la page
                                        if annotations_removed {
                                            if let Ok(page_dict_mut) = doc.get_object_mut(page_id) {
                                                if let pdf_modification::Object::Dictionary(ref mut dict_mut) = page_dict_mut {
                                                    dict_mut.set(b"Annots", pdf_modification::Object::Array(updated_annots));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Si des annotations ont été supprimées, sauvegarder le document
                        if annotations_removed {
                            files_with_removed_annotations += 1; // Incrémenter le compteur
                            let temp_path = pdf.path.with_extension("temp.pdf");
                            if let Err(e) = doc.save(&temp_path) {
                                eprintln!("Erreur lors de la sauvegarde temporaire : {}", e);
                                continue;
                            }

                            if let Err(e) = std::fs::rename(temp_path, &pdf.path) {
                                eprintln!("Erreur lors du remplacement du fichier PDF : {}", e);
                            } else {
                                println!("Annotations supprimées avec succès dans le fichier : {}", pdf.path.display());
                            }
                        } else {
                            println!("Aucune annotation correspondante trouvée dans le fichier : {}", pdf.path.display());
                        }
                    }

                    // Étape 7 : Afficher un message de succès si tout s'est bien passé
                    let message = if files_with_removed_annotations > 0 {
                        format!("Annotations supprimées dans {} fichier(s).", files_with_removed_annotations)
                    } else {
                        "Aucune annotation correspondante n'a été trouvée.".to_string()
                    };

                    if files_with_removed_annotations > 0 {
                        *self.show_annotation_removal_popup.lock().unwrap() = true; // Activer la popup
                        *self.annotation_removal_count.lock().unwrap() = files_with_removed_annotations; // Stocker le nombre de fichiers traités
                    
                        // Lancer un thread pour fermer la popup après 10 secondes
                        let ctx_clone = ctx.clone();
                        let show_annotation_removal_popup = Arc::clone(&self.show_annotation_removal_popup);
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_secs(10));
                            *show_annotation_removal_popup.lock().unwrap() = false;
                            ctx_clone.request_repaint(); // Repeindre l'interface pour actualiser l'état
                        });
                    }

                    *self.show_success_icon.lock().unwrap() = true;
                    println!("Popup : {}", message);
                    self.show_success_animation(ctx);
                    
                }
















                // ui.add_space(20.0); // Ajouter un espace vertical de 20 pixels
                // if ui.button("Réparer et remplacer les polices").clicked() {
                //     println!("Début de la vérification, réparation, et remplacement des polices...");

                //     // Vérifier si un dossier racine est sélectionné
                //     let root_selected = {
                //         let root_lock = self.root_folder.lock().unwrap();
                //         root_lock.is_some()
                //     };
                //     if !root_selected {
                //         *self.show_no_root_popup.lock().unwrap() = true;
                //         println!("Popup : Aucun dossier racine sélectionné.");
                //         return;
                //     }

                //     // Vérifier si des fichiers PDF sont trouvés
                //     let pdf_files_info = {
                //         let pdf_files_lock = self.pdf_files_info.lock().unwrap();
                //         pdf_files_lock.clone()
                //     };
                //     if pdf_files_info.is_empty() {
                //         *self.show_no_pdf_popup.lock().unwrap() = true;
                //         println!("Popup : Aucun fichier PDF trouvé pour la réparation.");
                //         return;
                //     }

                //     // Définir les chemins et noms de police
                //     let font_path = "assets/police/Roboto/Roboto-Medium.ttf"; // Chemin vers la police
                //     let font_name = "Roboto-Medium";

                //     for pdf in &pdf_files_info {
                //         println!("Vérification et traitement du fichier : {}", pdf.path.display());
                    
                //         // Charger ou réparer le document PDF
                //         let mut doc = match pdf_modification::Document::load(&pdf.path) {
                //             Ok(doc) => doc,
                //             Err(_) => {
                //                 println!("Le fichier {} est endommagé. Tentative de réparation...", pdf.path.display());
                //                 match repair_and_reload_pdf(&pdf.path) {
                //                     Ok(repaired_doc) => {
                //                         println!("Réparation réussie pour le fichier : {}", pdf.path.display());
                //                         repaired_doc
                //                     }
                //                     Err(e) => {
                //                         eprintln!("Échec de la réparation pour le fichier {} : {}", pdf.path.display(), e);
                //                         continue;
                //                     }
                //                 }
                //             }
                //         };

                //         // Obtenir le statut en appelant `verifier_polices`
                //         let status = verifier_polices(pdf.path.to_str().unwrap(), font_name);

                //         match status {
                //             Ok(info) => {
                //                 // Afficher le statut global du fichier PDF
                //                 println!("Statut global : {:?}", info.status);

                //                 // Boucle pour afficher les informations détaillées sur chaque police
                //                 println!("Détails des polices trouvées :");
                //                 for police in &info.polices_existantes {
                //                     println!(
                //                         "- Nom : {} | Intégrée : {} | Erreur : {}",
                //                         police.nom,
                //                         police.is_integrated,
                //                         police.erreur.as_deref().unwrap_or("Aucune")
                //                     );
                //                 }

                //                 // Vérification du statut global
                //                 match info.status {
                //                     PoliceStatus::PresenteEtValide => {
                //                         println!("La police est présente et valide.");
                //                     }
                //                     PoliceStatus::NonPresente => {
                //                         println!("La police n'est pas présente.");
                //                         // Ajoutez ici la logique pour ajouter la police
                //                     }
                //                     PoliceStatus::NonIntegree => {
                //                         println!("Problème détecté : la police n'est pas intégrée.");
                //                         for police in &info.polices_existantes {
                //                             if !police.is_integrated {
                //                                 println!(
                //                                     "Action nécessaire : Remplacement de la police non intégrée : {}",
                //                                     police.nom
                //                                 );
                //                                 // Ajoutez ici la logique de remplacement
                //                             }
                //                         }
                //                     }
                //                     PoliceStatus::FluxVideOuCorrompu => {
                //                         println!("Problème détecté : le flux de police est vide ou corrompu.");
                //                         for police in &info.polices_existantes {
                //                             if let Some(erreur) = &police.erreur {
                //                                 if erreur.contains("vide") || erreur.contains("corrompu") {
                //                                     println!(
                //                                         "Action nécessaire : Réparation du flux de la police : {}",
                //                                         police.nom
                //                                     );
                //                                 }
                //                             }
                //                         }
                //                     }
                //                     PoliceStatus::NonEnregistree => {
                //                         println!("La police est détectée mais non enregistrée.");
                //                         for police in &info.polices_existantes {
                //                             if police.erreur.as_deref() == Some("La police n'est pas enregistrée") {
                //                                 println!(
                //                                     "Action nécessaire : Enregistrement de la police : {}",
                //                                     police.nom
                //                                 );
                //                                 // Ajoutez ici la logique pour enregistrer la police
                //                             }
                //                         }
                //                     }
                //                     PoliceStatus::ErreurExtraction(flux_status) => {
                //                         match flux_status {
                //                             FluxStatus::InvalideGlyphes(reason) => {
                //                                 println!(
                //                                     "Erreur détectée : le flux contient des glyphes invalides. Détail : {}",
                //                                     reason
                //                                 );
                //                             }
                //                             FluxStatus::Illisible(reason) => {
                //                                 println!(
                //                                     "Erreur détectée : le flux est illisible. Détail : {}",
                //                                     reason
                //                                 );
                //                             }
                //                         }
                //                         // Ajoutez ici la logique pour gérer ces erreurs spécifiques
                //                     }
                //                     PoliceStatus::Erreur(e) => {
                //                         eprintln!("Une erreur a été rencontrée lors de la vérification : {}", e);
                //                     }
                //                 }
                //             }
                //             Err(e) => {
                //                 eprintln!("Erreur lors de la vérification des polices : {}", e);
                //             }
                //         }

                                       
                //     }


                //     // Récupérer le répertoire parent commun des fichiers PDF
                //     if let Some(common_dir) = pdf_files_info.first().and_then(|pdf| pdf.path.parent()) {
                //         if let Err(e) = clear_backup_files(common_dir) {
                //             eprintln!(
                //                 "Erreur lors de la gestion des fichiers de sauvegarde dans le répertoire {} : {}",
                //                 common_dir.display(),
                //                 e
                //             );
                //         }
                //     } else {
                //         eprintln!("Impossible de déterminer le répertoire parent des fichiers PDF.");
                //     }
                    


                //     println!("Fin de la vérification, réparation, et remplacement des polices.");
                //     *self.show_success_icon.lock().unwrap() = true;
                //     self.show_success_animation(ctx);
                // }











                










                // ui.add_space(5.0);  // Ajouter un espace vertical de 20 pixels
                // if ui.button("Ajouter la police").clicked() {
                //     println!("Début de l'ajout de la police...");

                //     // Vérifier si un dossier racine est sélectionné
                //     let root_selected = {
                //         let root_lock = self.root_folder.lock().unwrap();
                //         root_lock.is_some()
                //     };
                //     if !root_selected {
                //         *self.show_no_root_popup.lock().unwrap() = true;
                //         println!("Popup : Aucun dossier racine sélectionné.");
                //         return;
                //     }

                //     // Vérifier si des fichiers PDF sont trouvés
                //     let pdf_files_info = {
                //         let pdf_files_lock = self.pdf_files_info.lock().unwrap();
                //         pdf_files_lock.clone()
                //     };
                //     if pdf_files_info.is_empty() {
                //         *self.show_no_pdf_popup.lock().unwrap() = true;
                //         println!("Popup : Aucun fichier PDF trouvé pour l'ajout de police.");
                //         return;
                //     }

                //     // Ajouter la police
                //     let font_path = "assets/police/Roboto/Roboto-Medium.ttf"; // Chemin vers la police
                //     let font_name = "Roboto-Medium";

                //     for pdf in &pdf_files_info {
                //         println!("Traitement du fichier : {}", pdf.path.display());
                    
                //         // Appeler la fonction `verifier_polices` pour obtenir les informations sur la police
                //         let info = match verifier_polices(pdf.path.to_str().unwrap(), font_name) {
                //             Ok(i) => i, // Assignation du résultat à `info`
                //             Err(e) => {
                //                 eprintln!(
                //                     "Erreur lors de la vérification des polices pour le fichier {} : {}",
                //                     pdf.path.display(),
                //                     e
                //                 );
                //                 continue; // Passer au fichier suivant en cas d'erreur
                //             }
                //         };

                //         // Afficher les informations générales de la police
                //         println!(
                //             "Statut: {:?}, Polices détectées :",
                //             info.status
                //         );
                //         for police in &info.polices_existantes {
                //             println!(
                //                 "- Nom : {}, Intégrée : {}, Erreur : {}",
                //                 police.nom,
                //                 police.is_integrated,
                //                 police.erreur.as_deref().unwrap_or("Aucune")
                //             );
                //         }

                //         // Utiliser `info` pour déterminer l'action à effectuer
                //         match info.status {
                //             PoliceStatus::PresenteEtValide => {
                //                 println!(
                //                     "La police '{}' est présente et fonctionnelle dans le fichier : {}. Aucun ajout nécessaire.",
                //                     font_name,
                //                     pdf.path.display()
                //                 );
                //             }
                //             PoliceStatus::NonPresente => {
                //                 println!(
                //                     "La police '{}' n'est pas présente dans le fichier : {}. Ajout en cours...",
                //                     font_name,
                //                     pdf.path.display()
                //                 );
                //                 ajouter_police(&vec![pdf.clone()], font_path, font_name);
                //             }
                //             PoliceStatus::NonIntegree => {
                //                 println!(
                //                     "La police '{}' n'est pas intégrée dans le fichier : {}. Correction en cours...",
                //                     font_name,
                //                     pdf.path.display()
                //                 );
                //                 ajouter_police(&vec![pdf.clone()], font_path, font_name);
                //             }
                //             PoliceStatus::FluxVideOuCorrompu => {
                //                 println!(
                //                     "Le flux de la police '{}' est vide ou corrompu dans le fichier : {}. Correction en cours...",
                //                     font_name,
                //                     pdf.path.display()
                //                 );
                //                 ajouter_police(&vec![pdf.clone()], font_path, font_name);
                //             }
                //             PoliceStatus::NonEnregistree => {
                //                 println!(
                //                     "La police '{}' est détectée mais non enregistrée dans le fichier : {}. Enregistrement en cours...",
                //                     font_name,
                //                     pdf.path.display()
                //                 );
                //                 ajouter_police(&vec![pdf.clone()], font_path, font_name);
                //             }
                //             PoliceStatus::ErreurExtraction(flux_status) => {
                //                 println!(
                //                     "Erreur : Impossible d'extraire la police '{}' dans le fichier : {}.",
                //                     font_name,
                //                     pdf.path.display()
                //                 );
                //                 match flux_status {
                //                     FluxStatus::InvalideGlyphes(reason) => {
                //                         eprintln!("Glyphes invalides : {}", reason);
                //                     }
                //                     FluxStatus::Illisible(reason) => {
                //                         eprintln!("Flux illisible : {}", reason);
                //                     }
                //                 }
                //             }
                //             PoliceStatus::Erreur(e) => {
                //                 eprintln!(
                //                     "Une erreur a été rencontrée pour le fichier {} : {}",
                //                     pdf.path.display(),
                //                     e
                //                 );
                //             }
                //         }
                                                                
                          
                //     }
                //     println!("Fin de l'ajout de la police.");
                //     *self.show_success_icon.lock().unwrap() = true;
                //     self.show_success_animation(ctx);
                // }



















                

                
                








     








                
        });
        
    }
}
