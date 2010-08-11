import("etherpad.utils.*");
import("etherpad.log");
import("etherpad.control.pad.pad_control");
function onRequest() {
	var filter = request.path.toString().split("/specs/")[1].replace(/\//g,"-");
	renderHtml('groupIndex.ejs',{filter:filter},'supportGroupUrls');
	return true;
}
function render_page(){
	var padId = request.path.toString().split("/specs/")[1].replace(/\//g,"-");
	return pad_control.render_pad(padId);
}
