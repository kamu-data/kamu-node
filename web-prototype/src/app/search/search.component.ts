import {Component, HostListener, OnInit, ViewChild} from '@angular/core';
import {MatTableDataSource} from '@angular/material/table';
import {SearchApi} from "../api/search.api";
import {AppSearchService} from "./search.service";
import {SearchHistoryInterface} from "../interface/search.interface";
import { MatTable } from '@angular/material/table';
import AppValues from "../common/app.values";
import {searchAdditionalButtonsEnum} from "./search.interface";
import {SearchAdditionalButtonInterface} from "../components/search-additional-buttons/search-additional-buttons.interface";

const ELEMENT_DATA: SearchHistoryInterface[] = [];

@Component({
  selector: 'app-search',
  templateUrl: './search.component.html',
  styleUrls: ['./search.component.sass']
})
export class SearchComponent implements OnInit {

  public searchValue: string = '';
  public isMinimizeSearchAdditionalButtons: boolean = false;
  public searchAdditionalButtonsData: SearchAdditionalButtonInterface[] = [{
    textButton: searchAdditionalButtonsEnum.Descission
  }, {
    textButton: searchAdditionalButtonsEnum.Reputation
  }, {
    textButton: searchAdditionalButtonsEnum.Explore,
    styleClassContainer: 'app-active-button__container',
    styleClassButton: 'app-active-button'
  }, {
    textButton: searchAdditionalButtonsEnum.DeriveForm,
    styleClassContainer: 'app-active-button__container',
    styleClassButton: 'app-active-button'
  }];

  public displayedColumns: string[] = [];
  public dataSource = new MatTableDataSource<any>(ELEMENT_DATA);

  @HostListener('window:resize', ['$event'])
  private checkWindowSize(): void {
    this.isMinimizeSearchAdditionalButtons = (window.innerWidth < window.innerHeight);
  }
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


  public onClickSearchAdditionalButton(method: string) {
    if (method === searchAdditionalButtonsEnum.DeriveForm) {
      this.onClickDeriveForm();
    }
    if (method === searchAdditionalButtonsEnum.Reputation) {
      this.onClickReputation();
    }
    if (method === searchAdditionalButtonsEnum.Explore) {
      this.onClickExplore();
    }
    if (method === searchAdditionalButtonsEnum.Descission) {
      this.onClickDescission();
    }
  }

  private onClickDeriveForm() {
  }
  private onClickExplore() {
  }
  private onClickReputation() {
  }
  private onClickDescission() {
  }

  private renderTable(data: SearchHistoryInterface[]): void {
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
    this.appSearchService.search();
  }
  public onSearchProjections(): void {
    this.appSearchService.searchLastTenFields();
  }
  public onSearchMetadata(): void {
    this.appSearchService.onSearchMetadata();
  }

}