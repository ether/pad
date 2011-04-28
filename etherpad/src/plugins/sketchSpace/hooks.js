import("etherpad.log");
import("faststatic");
import("etherpad.utils.*");
import("etherpad.globals.*");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");
import("etherpad.helpers");

function docbarItemsPad() {
    return ['<a href="javascript:sketchSpace.insertImage()" title="SketchSpace"><img src="/static/html/plugins/sketchSpace/editbar_icon.png">New blank SketchSpace</a>',
	    '<a href="javascript:void(0)" title="SketchSpace upload PDF" id="sketchSpaceAddPdfImage"><img src="/static/html/plugins/sketchSpace/editbar_pdf_icon.png">New SketchSpace(s) from PDF</a>'];
}

function editBarItemsLeftPad(arg) {
  return arg.template.include('sketchSpaceEditbarButtons.ejs', undefined, ['sketchSpace']);
}

function editBarItemsLeftPadView(arg) {
  helpers.includeCss("plugins/sketchSpace/ace.css");
  helpers.includeJs("plugins/sketchSpace/ace_inner.js");
}


function modals(arg) {
  return arg.template.include('sketchSpaceModals.ejs', undefined, ['sketchSpace']);
}
