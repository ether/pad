import("etherpad.log");
import("etherpad.utils.*");
import("etherpad.collab.server_utils.padIdToReadonly");

function docbarItemsPad() {
    var padId = request.path.split("/")[1];
    return ["<a href='/ep/pad/view/" +
            padIdToReadonly(padId) +
            "/latest'><img src='/static/img/plugins/linkReadOnly/glasses.gif'>Read-only Version</a>"];
}
