import {
  Component,
  EventEmitter,
  Input,
  OnInit,
  Output
} from "@angular/core";
import {SearchOverviewInterface} from "../../interface/search.interface";
import AppValues from "../../common/app.values";
import * as moment from 'moment-timezone';
import {Moment} from "moment-timezone";
@Component({
  selector: 'app-repo-list',
  templateUrl: './repo-list.component.html',
  styleUrls: ['./repo-list.sass']
})
export class RepoListComponent implements OnInit {
  @Input() public dataSource: SearchOverviewInterface[];
  @Input() public resultUnitText: string;
  @Input() public isResultQuantity?: boolean = false;
  @Input() public isClickableRow?: boolean = false;
  @Output() public onSelectDatasetEmit: EventEmitter<string> = new EventEmitter();

  constructor() { }
  public ngOnInit(): void {
  }
  public momentConverDatetoLocalWithFormat(date: string): string {
    return AppValues.momentConverDatetoLocalWithFormat({date: new Date(date), format: 'DD MMM YYYY', isTextDate: true});
  }
  public onSelectDataset(id: string): void {
    this.onSelectDatasetEmit.emit(id);
  }

  public searchResultQuantity(dataSource: SearchOverviewInterface[] = []): string {
      if(!Array.isArray(dataSource)) {
        return '0';
      }
      return dataSource.length.toString();
  }

}