import { Component, OnInit, ViewChild } from '@angular/core';
import {BehaviorSubject, Observable} from "rxjs";
import {DataSource} from "@angular/cdk/collections";
import {MatTableDataSource} from '@angular/material/table';
import {SearchApi} from "../api/search.api";
import {AppSearchService} from "./search.service";
import {SearchHistoryInterface} from "../interface/search.interface";
import { MatTable } from '@angular/material/table';
import AppValues from "../common/app.values";

const ELEMENT_DATA: SearchHistoryInterface[] = [];

@Component({
  selector: 'app-search',
  templateUrl: './search.component.html',
  styleUrls: ['./search.component.sass']
})
export class SearchComponent implements OnInit {

  public searchValue: string = '';
  displayedColumns: string[] = [];
  dataSource = new MatTableDataSource<any>(ELEMENT_DATA);

  constructor(
      private appSearchService: AppSearchService
  ) { }

  public ngOnInit(): void {
    this.onSearch();
    this.appSearchService.onSearchChanges.subscribe((value: string) => {
      this.searchValue = value;
    })

    this.appSearchService.onSearchDataChanges.subscribe((data: any[]) => {
      this.renderTable(data);
    });
  }

  private renderTable(data: SearchHistoryInterface[]): void {
    debugger
    const elementsData: SearchHistoryInterface[] = [];
    if (!data.length) {
      this.dataSource.data = [];
    }
    this.dataSource.data = [];
    const keys_data: string[] = Object.keys(data[0]);

    this.displayedColumns = keys_data;

    const dataSource = this.dataSource.data;
    data.forEach((field: SearchHistoryInterface) => {
      dataSource.push(field);
    })
    this.dataSource.data = dataSource;
  }

  public changeColumnName(columnName: string): string {
    columnName = columnName.replace('_', ' ');
    return AppValues.capitalizeFirstLetter(columnName);
  }

  public showHistory(): void {
    this.appSearchService.searchHistory();
  }

  public onSearch(): void {
    debugger
    this.appSearchService.search();
  }
  public onSearchProjections(): void {
    this.appSearchService.searchLastTenFields();
  }
  public onSearchMetadata(): void {
    this.appSearchService.onSearchMetadata();
  }

}