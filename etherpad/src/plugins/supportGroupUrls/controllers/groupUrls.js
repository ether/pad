import("etherpad.utils.*");

function render_index(gubbins) {
	renderHtml('groupIndex.ejs',{gubbins:gubbins},'supportGroupUrls');
	return true;
}