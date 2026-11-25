const dialog = document.getElementById("video");
const video = document.getElementById("vid");

function openVideo() {
  dialog.showModal();
  video.currentTime = 0;
  video.play().catch(() => {});
}

function closeVideo() {
  video.pause();
  dialog.close();
}

document.getElementById("play").addEventListener("click", openVideo);
document.getElementById("poster").addEventListener("click", openVideo);
dialog.querySelector(".close").addEventListener("click", closeVideo);
dialog.addEventListener("click", (e) => { if (e.target === dialog) closeVideo(); });
dialog.addEventListener("close", () => video.pause());

document.querySelectorAll("[data-copy]").forEach((btn) => {
  btn.addEventListener("click", async () => {
    try {
      await navigator.clipboard.writeText(btn.dataset.copy);
      btn.textContent = "Copied";
    } catch {
      btn.textContent = "Press Ctrl+C";
    }
    setTimeout(() => { btn.textContent = "Copy"; }, 1600);
  });
});

const nav = document.querySelector(".nav");
addEventListener("scroll", () => nav.classList.toggle("scrolled", scrollY > 8), { passive: true });

const THEME_TOML = {
  mocha: ["catppuccin", "mocha"], latte: ["catppuccin", "latte"], rose: ["rose-pine", "main"],
  dawn: ["rose-pine", "dawn"], tokyo: ["tokyo-night", "night"],
};
const themer = document.getElementById("themer");
themer.querySelectorAll("[data-t]").forEach((btn) => {
  btn.addEventListener("click", () => {
    themer.querySelectorAll("[data-t]").forEach((b) => b.setAttribute("aria-selected", String(b === btn)));
    themer.dataset.themeName = btn.dataset.t;
    const [name, flavor] = THEME_TOML[btn.dataset.t];
    document.getElementById("theme-name").textContent = `"${name}"`;
    document.getElementById("theme-flavor").textContent = `"${flavor}"`;
  });
});

document.querySelectorAll("[data-s]").forEach((btn) => {
  btn.addEventListener("click", () => {
        const page = btn.dataset.s;
    document.getElementById("setsrc").srcset = `assets/settings/${page}-mocha.webp`;
    document.getElementById("setimg").src = `assets/settings/${page}-latte.webp`;
  });
});
