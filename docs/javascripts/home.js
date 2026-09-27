// Copy button for the landing page's install box (docs/overrides/home.html).
// Uses one delegated listener so it keeps working with Material's instant
// navigation, which swaps page content without reloading scripts.
document.addEventListener("click", (event) => {
  const button = event.target.closest(".rd-copy");
  if (!button || !navigator.clipboard) return;

  navigator.clipboard.writeText(button.dataset.copy).then(() => {
    const label = button.querySelector("span");
    label.textContent = "Copied";
    button.classList.add("rd-copy--done");
    setTimeout(() => {
      label.textContent = "Copy";
      button.classList.remove("rd-copy--done");
    }, 1500);
  });
});
