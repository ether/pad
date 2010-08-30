import("etherpad.utils.*");
import("etherpad.log");
import("etherpad.control.pad.pad_control");
import("sqlbase.sqlobj");
import("sqlbase.sqlbase");
import("etherpad.pad.exporthtml");
import("etherpad.pad.model");
import("etherpad.pad.padutils");
import("etherpad.sessions.getSession");
import("plugins.padHierarchy.helpers.hierarchyHelper.*");

function onRequest() {
	var section_path = (request.path.toString() == '/pads') ? 'pads'  :  request.path.toString().split("/pads/")[1].replace(/\/$/ , '');
	var id_filter = section_path=='pads' ? '' : section_path.replace(/\//g,"-");

	var grouped_pads = getPadsBelow(id_filter);
 
	
	renderHtml('hierarchyIndex.ejs',
		   {path:request.path,
		    grouped_pad_list:getGroupChildren(grouped_pads),
		    pads:grouped_pads,
		    selected_pad:grouped_pads},
		   ['padHierarchy']);
	return true;
}

function edit_page(){
	var padId = request.path.toString().split("/pads/")[1].replace(/\/\+edit$/, '').replace(/\//g,"-");
	getSession().instantCreate = encodeURIComponent(padId);
	
	return pad_control.render_pad(padId);
}

function redirect_to_pads_path(){
	if (!isStaticRequest()) {
		if (request.path == '/pads') {
			return onRequest();
		} else {
			response.redirect("/pads" + request.path);
		}
	} else {
		// do something else... this static routing's a bit strange..
	}
}

function render_main(){
	response.redirect("/pads");
}
