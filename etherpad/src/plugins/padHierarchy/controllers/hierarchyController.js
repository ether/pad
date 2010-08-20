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

	var matching_pads = sqlobj.selectMulti("PAD_SQLMETA",{id:['like', id_filter+'%']});
	
	var grouped_pads = getHierarchy(matching_pads.map(function(item){ return item.id; }),id_filter, function(pad){
		var json = sqlbase.getJSON("PAD_META", pad.id);
		if (json) {
			pad.meta = json;
			var html = padutils.accessPadLocal(pad.id, function(pad){
				return pad.exists() ? exporthtml.getPadHTML(pad) : null;
			}, 'r');
			pad.html = html;
		}
		log.info("\n\n" + id_filter + "?=" + pad.id   );
		return pad;
	});
 
	
	renderHtml('hierarchyIndex.ejs',{	
									grouped_pad_list:getGroupChildren(grouped_pads),
									pads:grouped_pads,
									selected_pad:grouped_pads}
									,'padHierarchy');
	return true;
}
function getGroupLink(group){
	return '<a href="'+ group.path +'" >' + (group.shortName || group.id || 'pads') +'</a>';
}

function getGroupChildren( group ){
	if(group.children.length == 0){
		return '<li>' +getGroupLink(group) + "</li>";
	}
	var result = getGroupLink(group) + '<ul>';
	for(var i=0; i<group.children.length; i++){
		var child = group.children[i];
		result += getGroupChildren(child);
	}
	return result+ '</ul>';
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
		}
		else {
			response.redirect("/pads" + request.path);
		}
	}else{
		// do something else... this static routing's a bit strange..
	}
}
function render_main(){
	response.redirect("/pads");
}
