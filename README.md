# tp_dithering

### Question 1)

cargo new tp_dithering

cargo add argh
cargo add image=0.24

cargo build

### Question 2)

Pour ouvrir une image depuis un fichier, on utilise 
```image::open(path_in)?;``` 

On obtient un DynamicImage, à quoi correspond ce type?

On peut obtenir une image en mode rbg8 en utilisant la méthode to_rgb8() sur l'image obtenue.


### Question 3)

Si l'image de départ avait un canal alpha, on peut utiliser la méthode to_rgb8() pour obtenir une image en mode rbg8 sans canal alpha.

Expliquer dans le README de votre rendu ce qui se passe ici.

### Question 5)

L'image est reconnaissable de loin malgré qu'on voit les carrés blanc et de près on voit mieux les carrés blancs.

### Question 6)

Pour obtenir la luminosité d'un pixel, on peut utiliser la méthode to_luma().0[0] sur un pixel.

### Question 9)

Pour calculer la distance entre deux couleurs, on peut utiliser la formule de la distance euclidienne.
Formule:

```sqrt((r1-r2)^2 + (g1-g2)^2 + (b1-b2)^2)```

### Question 11)

Si on donne une palette vide à notre application, on ne peut pas appliquer de dithering car on ne peut pas trouver la couleur la plus proche de chaque pixel. On peut donc afficher un message d'erreur à l'utilisateur pour lui dire que la palette est vide et qu'il doit la remplir pour pouvoir appliquer le dithering. Minimum 2 couleurs sont demandé.

### Question 21)

```
Usage: tp_dithering input [output] command [args]

Convertit une image en monochrome ou vers une palette réduite de couleurs.

Arguments positionnels :
  input             le fichier d'entrée
  output            le fichier de sortie (optionnel)

Options :
  --help, help      afficher ce message d'aide

Commandes :
  seuil             Convertit l'image en monochrome par seuillage.
  palette           Convertit l'image en utilisant une palette de couleurs limitée.
  tramage           Convertit l'image en utilisant une méthode de tramage.
```

Commande utilisable : 
```
- cargo run -- ./image/iut.jpeg ../image/out.jpeg seuil

- cargo run -- ./image/iut.jpeg ./image/question7.png seuil --couleur-bas bleu  --couleur-haut rouge

- cargo run -- ./image/iut.jpeg ./image/question10.png palette --n-couleurs 8

- cargo run -- ./image/iut.jpeg ./image/question12.png tramage
```

### Question 22)

Pour représenter les options fournies par l'utilisateur, le type Rust suivant est utilisé :

La structure principale DitherArgs regroupe les arguments communs comme le fichier d'entrée et de sortie.
Mode permet de gérer plusieurs sous-commandes (seuil, palette, tramage).
Chaque sous-commande a sa propre structure pour définir ses options spécifiques.
Voici le code du type Rust correspondant :

```rust
#[derive(Debug, Clone, PartialEq, FromArgs)]
/// Convertit une image en monochrome ou vers une palette rÃ©duite de couleurs.
struct DitherArgs {

    /// le fichier dâentrÃ©e
    #[argh(positional)]
    input: String,

    /// le fichier de sortie (optionnel)
    #[argh(positional)]
    output: Option<String>,

    /// le mode dâopÃ©ration
    #[argh(subcommand)]
    mode: Mode
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand)]
enum Mode {
    Seuil(OptsSeuil),
    Palette(OptsPalette),
    Tramage(OptsTramage)
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="seuil")]
/// Rendu de lâimage par seuillage monochrome.
struct OptsSeuil {
    
    /// la couleur pour les pixels en dessous du seuil (optionnel, par défaut noir)
    #[argh(option, default = "\"noir\".to_string()")]
    couleur_bas: String,

    /// la couleur pour les pixels au-dessus du seuil (optionnel, par défaut blanc)
    #[argh(option, default = "\"blanc\".to_string()")]
    couleur_haut: String,
}


#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="palette")]
/// Rendu de lâimage avec une palette contenant un nombre limitÃ© de couleurs
struct OptsPalette {

    /// le nombre de couleurs Ã  utiliser, dans la liste [NOIR, BLANC, ROUGE, VERT, BLEU, JAUNE, CYAN, MAGENTA]
    #[argh(option)]
    n_couleurs: usize
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="tramage")]
/// Rendu de lâimage par tramage
struct OptsTramage {
}
```

### Question 23)

Implémenté ici : [main](src/main.rs)