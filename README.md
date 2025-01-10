
# 🌟 Outil d'annotation de PDF récursif


![Rust Badge](https://img.shields.io/badge/Rust-%23f74c00?style=flat&logo=rust&logoColor=white)
![License](https://img.shields.io/badge/license-GNU2-blue)

Ce programme est une application graphique écrite en Rust. 
I'application permet d'ajouter des `annotations` à des fichiers PDF de manière interactive et `récursif`.

## Compatibilité

![Windows Badge](https://img.shields.io/badge/Windows-%230078D6?style=flat&logo=windows&logoColor=white)


---

## 🚀 Fonctionnalités

1. **📂 Sélection et gestion des fichiers PDF** :
   - Parcourt un dossier pour trouver tous les fichiers PDF.
   - Affiche une liste des fichiers trouvés.

2. **🔍 Prévisualisation et annotation** :
   - Ajoute du texte personnalisé à des positions spécifiques des fichiers PDF.
   - Supporte différents formats de pages, comme A4, A5 et US Letter.
   - Permet d'appliquer les annotations à toutes les pages ou uniquement à la première page de manière récursif sur les PDF.

3. **🎨 Personnalisation des annotations** :
   - Texte personnalisable.
   - Couleurs variées pour les annotations.
   - Gestion des positions avec ajustements dynamiques selon le format de la page.

4. **📝 Création de nouveaux fichiers PDF** :
   - Génère de nouveaux fichiers PDF à partir de zéro avec un text pérsonalisable.

5. **🔧 Réparation de fichiers PDF corrompus** :
   - Intègre des outils comme `QPDF` pour réparer les fichiers PDF et vérifier leur intégrité.

6. **✒️ Manipulation avancée des polices** :
   - Ajout ou remplacement de polices dans les fichiers PDF.
   - Vérification et gestion des polices intégrées.



---

## 🛠️ Compilation

### 📋 Prérequis
- Rust et Cargo installés.
- Les bibliothèques suivantes doivent être ajoutées à votre projet :


### 🖥️ Commandes - Powershell
0. Installation de Rust et Cargo
    ```powershell
    Invoke-WebRequest -Uri "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe" -OutFile rustup-init.exe; 
    .\rustup-init.exe -y

    ```
1. Clonez le projet :
   ```powershell
   git clone https://github.com/00MY00/Rusty_PDF_Manager
   cd Rusty_PDF_Manager
   ```

2. Compilez et lancez :
   ```powershell
   cargo build --release
   ```

---



## 🎯 Utilisation

### Interface
- **📁 Sélection du dossier racine** : Permet de parcourir vos fichiers pour trouver des PDF.
- **🔍 Prévisualisation** : Montre un aperçu du fichier PDF avec les annotations à appliquer.
- **⚙️ Options d'annotation** :
  - Définissez le texte et la couleur.
  - Spécifiez les positions d'annotation.

### Création ou modification de PDF
- Cliquez sur **Nouveau Fichier PDF** pour générer un fichier à partir de zéro.
- Utilisez **Annotation Récursive** pour ajouter du texte à tous les fichiers PDF sélectionnés.

---

## ⚠️ Limitations connues
- La gestion des polices non intégrées peut rencontrer des limitations selon les fichiers.
- Certains formats PDF complexes peuvent ne pas être entièrement compatibles.

---

## 🖋️ Contributeurs

- **00MY00** : Auteur principal.

---

### 🖱️ Lien de téléchargement
[![Télécharger le code](https://img.shields.io/badge/Télécharger-Code-blue?style=for-the-badge)](https://github.com/00MY00/Rusty_PDF_Manager/raw/refs/heads/main/Compiled/Windows/Rusty_PDF_Manager%20-%20Windows.7z)
