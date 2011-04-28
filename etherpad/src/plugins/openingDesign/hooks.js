import("etherpad.log");
import("faststatic");
import("etherpad.utils.*");
import("etherpad.globals.*");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");
import("etherpad.helpers");

function docbarItemsPad() {
    return ['<a href="javascript:sketchSpace.insertImage()" title="SketchSpace"><img src="/static/html/plugins/sketchSpace/editbar_icon.png">New blank SketchSpace</a>',
	    '<a href="javascript:void(0)" title="SketchSpace upload PDF" class="sketchSpaceAddPdfImage"><img src="/static/html/plugins/sketchSpace/editbar_pdf_icon.png">New SketchSpace(s) from PDF</a>',
	    '<a href="javascript:void((function () { var e = jQuery.Event(\'mousedown\'); e.pageX = 0; $(\'#vdraggie\').trigger(e); e = jQuery.Event(\'mouseup\'); e.pageX = 0; $(\'#vdraggie\').trigger(e); })())" title="Share answer with...">Share answer with...</a>',];
}

function modals(arg) {
 return '<script src="/static/js/plugins/openingDesign/fixui.js"></script>' +
        '<link rel="stylesheet" href="/static/css/plugins/openingDesign/editor.css"/>';
}
