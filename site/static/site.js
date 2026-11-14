// Terminal of Tinsel's only script. The pages work without it: it adds a
// copy button to each block of terminal text and, in December, opens
// tonight's door on the advent calendar.

(function () {
  "use strict";

  function copyButtons() {
    if (!navigator.clipboard) {
      return;
    }
    document.querySelectorAll(".code").forEach(function (block) {
      var button = block.querySelector(".copy");
      var text = block.querySelector("code");
      if (!button || !text) {
        return;
      }
      button.hidden = false;
      button.addEventListener("click", function () {
        navigator.clipboard.writeText(text.textContent).then(function () {
          button.textContent = "Copied";
          setTimeout(function () {
            button.textContent = "Copy";
          }, 1500);
        });
      });
    });
  }

  function tonightsDoor() {
    var now = new Date();
    if (now.getMonth() !== 11 || now.getDate() > 24) {
      return;
    }
    var door = document.querySelector('.door[data-night="' + now.getDate() + '"]');
    if (door) {
      door.open = true;
      door.classList.add("tonight");
    }
  }

  copyButtons();
  tonightsDoor();
})();
