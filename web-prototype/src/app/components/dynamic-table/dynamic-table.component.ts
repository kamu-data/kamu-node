import {
  AfterContentInit,
  Component,
  EventEmitter,
  Input,
  OnChanges,
  OnInit,
  Output,
  SimpleChanges
} from "@angular/core";
import {MatTableDataSource} from "@angular/material/table";
import AppValues from "../../common/app.values";

const ELEMENT_DATA: any[] = [];
@Component({
  selector: 'app-dynamic-table',
  templateUrl: './dynamic-table.component.html',
  styleUrls: ['./dynamic-table.sass']
})
export class DynamicTableComponent implements OnInit, OnChanges, AfterContentInit {
  @Input() public isTableHeader: boolean;
  @Input() public tableColumns?: any[];
  @Input() public tableSource: any[];
  @Input() public isResultQuantity?: boolean = false;
  @Input() public resultUnitText: string;
  @Input() public isClickableRow: boolean = false;
  @Output() public onSelectDatasetEmit: EventEmitter<string> = new EventEmitter();

  public dataSource = new MatTableDataSource<any>(ELEMENT_DATA);
  public displayedColumns: string[] = [];

  constructor() { }
  public ngOnInit(): void {
    this.tableSource && this.renderTable(this.tableSource);
  }
  public ngOnChanges(changes: SimpleChanges): void {
    this.tableSource && this.renderTable(this.tableSource);
  }

  public ngAfterContentInit(): void {
    this.tableSource && this.renderTable(this.tableSource);
  }

  public changeColumnName(columnName: string): string {
    columnName = columnName.replace('_', ' ');
    return AppValues.capitalizeFirstLetter(columnName);
  }

  public onSelectDataset(dataset: any): void {
    this.onSelectDatasetEmit.emit(dataset);
  }

  private renderTable(data: any[]): void {
    if (!data.length) {
      this.dataSource.data = [];
      return;
    }
    this.dataSource.data = [];
    this.displayedColumns = Object.keys(data[0]);

    const dataSource = this.dataSource.data;
    data.forEach((field: any) => {
      dataSource.push(field);
    })
    this.dataSource.data = dataSource;
    this.dataSource = new MatTableDataSource(dataSource);
  }

  public searchResultQuantity(tableSource: any[]): string {
      return tableSource.length.toString();
  }
}