// This is a hack to get around ACEs brain-dead limit on onClick on
// links inside the ACE domlines...

$(document).ready(function () {
  $("body").click(function (event) {
    if ($(event.target).filter(".sketchSpaceImageLink").length > 0) {
      top.sketchSpace.imageLinkClicked(event.target.parentNode);
    }
  });
});

