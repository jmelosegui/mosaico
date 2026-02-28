// Rename "Coal" to "Dark" in the theme picker.
document.addEventListener("DOMContentLoaded", function () {
    var coal = document.getElementById("mdbook-theme-coal");
    if (coal) {
        coal.textContent = "Dark";
    }
});
