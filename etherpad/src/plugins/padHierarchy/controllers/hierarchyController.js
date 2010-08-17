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
	var section = request.path.toString().split("/specs/")[1];
	var filter = section.replace(/\//g,"-");
	function shortName(pad_id){
		return pad_id.replace(filter, '');
	}
	function groupBasedId(pad_id){
		return request.path + shortName(pad_id);	
	}
	
	var matching_pads = sqlobj.selectMulti("PAD_SQLMETA",{id:['like', filter+'%']});
	var pads = {};
	var grouped_pads = getHierarchy(matching_pads.map(function(item){ return item.id; }),section.replace(/^\/|\/$/g, ''));
	
	// seems like too many db queries - is there a selectMultiJSON command thing? getAllJSON? Does that support filter conditions?
	for (var i in matching_pads) {
		var json= sqlbase.getJSON("PAD_META", matching_pads[i].id);
		
		if(json) pads[matching_pads[i].id] = json;
		var html = padutils.accessPadLocal(matching_pads[i].id, function(pad){
			return pad.exists() ? exporthtml.getPadHTML( pad ) : null;	
		}, 'r');			
		pads[matching_pads[i].id].html = html;
	}
	
	var summary_pad_id = request.path.replace(/^\/specs\//,'').replace(/\/$/,'').replace(/\//g, '-');
	
	var summary = 
	padutils.accessPadLocal(summary_pad_id, function(pad){
		return pad.exists() ? exporthtml.getPadHTML( pad ) : null;	
	}, 'r');
	 
	
	renderHtml('hierarchyIndex.ejs',{	
									grouped_pad_list:getGroupChildren(grouped_pads),
									summary_pad_id:summary_pad_id,
									summary:summary,
									section:section,
									filter:filter, 
									pads:pads,
									grouped_pads:grouped_pads,
									groupBasedId:groupBasedId,
									shortName:shortName},'padHierarchy');
	return true;
}
function getGroupLink(group){
	return '<a href="'+ group.path +'" >' + (group.shortName || group.id) + '</a>';
}

function getGroupChildren( group ){
	if(group.children.length == 0){
		return '<li>' +getGroupLink(group);
	}
	var result = getGroupLink(group) + '<ul>';
	for(var i=0; i<group.children.length; i++){
		var child = group.children[i];
		result += getGroupChildren(child);
	}
	return result+ '</ul>';
}
		
		


function render_page(){
	var padId = request.path.toString().split("/specs/")[1].replace(/\//g,"-");
	getSession().instantCreate = encodeURIComponent(padId);
	
	return pad_control.render_pad(padId);
}
function redirect_to_specs_path(){
	if (!isStaticRequest()) {
		response.redirect("/specs" + request.path);
	}else{
		// do something else... this static routing's a bit of a mess..
	}
}
function render_main(){
	response.redirect("/specs/");
}
