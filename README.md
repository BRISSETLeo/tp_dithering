# tp_dithering

Question 1)

cargo new tp_dithering

cargo add argh
cargo add image=0.24

cargo build

Question 2)

Pour ouvrir une image depuis un fichier, on utilise 
```image::open(path_in)?;``` 

On obtient un DynamicImage, à quoi correspond ce type?
```DynamicImage est une énumération qui permet de représenter une image de n'importe quel type.```

On peut obtenir une image en mode rbg8 en utilisant la méthode to_rgb8() sur l'image obtenue.


Question 3)

Si l'image de départ avait un canal alpha, on peut utiliser la méthode to_rgb8() pour obtenir une image en mode rbg8 sans canal alpha.

Expliquer dans le README de votre rendu ce qui se passe ici.


Question 9)
Comment calculer la distance entre deux couleurs? Indiquer dans le README la méthode de
calcul choisie.

```La distance entre deux couleurs peut être calculée en utilisant la formule de la distance euclidienne.```

