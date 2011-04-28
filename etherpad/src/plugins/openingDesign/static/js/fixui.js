$(document).ready(function () {
  var e = jQuery.Event('mousedown');
  e.pageX = 0; $('#vdraggie').trigger(e);
  e = jQuery.Event('mouseup');
  e.pageX = 0; $('#vdraggie').trigger(e);
});
