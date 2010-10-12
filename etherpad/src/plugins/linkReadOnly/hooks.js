import("etherpad.log");
import("etherpad.utils.*");
import("etherpad.collab.server_utils");
import("etherpad.pad.padutils");

function docbarItemsPad() {
    var padId = request.path.split("/")[1];
    return ["<a href='/ep/pad/view/" +
            server_utils.padIdToReadonly(padutils.getGlobalPadId(padId)) +
            "/latest'><img src='/static/img/plugins/linkReadOnly/glasses.gif'>Read-only Version</a>"];
}
