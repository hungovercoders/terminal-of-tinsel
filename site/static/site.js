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
        var say = function (word) {
          button.textContent = word;
          setTimeout(function () {
            button.textContent = "Copy";
          }, 2500);
        };
        navigator.clipboard.writeText(text.textContent).then(
          function () {
            say("Copied");
          },
          function () {
            // The browser refused: select the text so it can be copied by hand.
            var range = document.createRange();
            range.selectNodeContents(text);
            var selection = window.getSelection();
            selection.removeAllRanges();
            selection.addRange(range);
            say("Selected: press Ctrl or Cmd + C");
          }
        );
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
