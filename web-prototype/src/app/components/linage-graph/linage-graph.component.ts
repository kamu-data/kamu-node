import {Component, Input,} from "@angular/core";
import {Edge} from "@swimlane/ngx-graph/lib/models/edge.model";
import {Node} from "@swimlane/ngx-graph/lib/models/node.model";
import { MiniMapPosition } from "ngx-graph/projects/swimlane/ngx-graph/src/lib/enums/mini-map-position.enum";

@Component({
  selector: 'app-linage-graph',
  templateUrl: './linage-graph.component.html',
})
export class LinageGraphComponent {
  @Input() public view: [number, number];
  @Input() public links: Edge[];
  @Input() public nodes: Node[];

  public draggingEnabled: boolean = false;
  public panningEnabled: boolean = true;
  public zoomEnabled: boolean = true;
  public zoomSpeed: number = 0.1;
  public minZoomLevel: number = 0.1;
  public maxZoomLevel: number = 4.0;
  public panOnZoom: boolean = true;
  public autoZoom: boolean = true;
  public autoCenter: boolean = false;
  public showMiniMap: boolean = true;
  public miniMapPosition: MiniMapPosition = MiniMapPosition.UpperLeft;
}