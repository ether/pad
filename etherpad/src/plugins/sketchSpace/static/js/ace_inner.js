// This is a hack to get around ACEs brain-dead limit on onClick on
// links inside the ACE domlines...

$(document).ready(function () {
  $("body").click(function (event) {
    var target = undefined;
    if ($(event.target).filter(".sketchSpaceImageLink").length > 0) {
      target = $(event.target).filter(".sketchSpaceImageLink")[0];
    } else if ($(event.target).find(".sketchSpaceImageLink").length > 0) {
      target = $(event.target).find(".sketchSpaceImageLink")[0];
    }
    if (target)
      parent.parent.sketchSpace.imageLinkClicked(target.parentNode);
  });
});

