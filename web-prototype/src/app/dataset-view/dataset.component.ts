import {AfterContentInit, Component, HostListener, OnDestroy, OnInit, ViewChild} from '@angular/core';
import {
    DatasetInfoInterface, DatasetKindInterface, DatasetKindTypeNames,
    PageInfoInterface,
    SearchHistoryInterface, SearchOverviewDatasetsInterface,
    SearchOverviewInterface
} from '../interface/search.interface';
import AppValues from '../common/app.values';
import {SearchAdditionalButtonInterface} from '../components/search-additional-buttons/search-additional-buttons.interface';
import {MatSidenav} from '@angular/material/sidenav';
import {SideNavService} from '../services/sidenav.service';
import {searchAdditionalButtonsEnum} from '../search/search.interface';
import {DatasetViewTypeEnum} from './dataset-view.interface';
import {AppDatasetService} from './dataset.service';
import {NavigationEnd, Router} from '@angular/router';
import {Edge} from '@swimlane/ngx-graph/lib/models/edge.model';
import {ClusterNode, Node} from '@swimlane/ngx-graph/lib/models/node.model';
import {filter} from 'rxjs/operators';
import {ModalService} from '../components/modal/modal.service';


@Component({
    selector: 'app-dataset',
    templateUrl: './dataset.component.html',
    styleUrls: ['./dataset-view.component.sass']
})
export class DatasetComponent implements OnInit, AfterContentInit, OnDestroy {

    @ViewChild('sidenav', {static: true}) public sidenav?: MatSidenav;
    public isMobileView = false;
    public datasetInfo: DatasetInfoInterface;
    public searchValue = '';
    public currentPage: number;
    public isMinimizeSearchAdditionalButtons = false;
    public datasetViewType: DatasetViewTypeEnum = DatasetViewTypeEnum.overview;
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

    /* eslint-disable  @typescript-eslint/no-explicit-any */
    public tableData: {
        isTableHeader: boolean,
        displayedColumns?: any[],
        tableSource: any,
        isResultQuantity: boolean,
        isClickableRow: boolean,
        pageInfo: PageInfoInterface,
        totalCount: number
    };
    public searchData: SearchOverviewDatasetsInterface[] | SearchHistoryInterface [] = [];

    public linageGraphView: [number, number] = [500, 600];
    public linageGraphLink: Edge[] = [];
    public linageGraphNodes: Node[] = [];
    public linageGraphClusters: ClusterNode[] = [];
    public isAvailableLinageGraph = false;


    private _window: Window;

    @HostListener('window:resize', ['$event'])
    private checkWindowSize(): void {
        this.isMinimizeSearchAdditionalButtons = AppValues.isMobileView();
        this.isMobileView = AppValues.isMobileView();

        if (AppValues.isMobileView()) {
            this.sidenavService.close();
        } else {
            this.sidenavService.open();
        }
        this.changeLinageGraphView();
    }

    constructor(
        private appDatasetService: AppDatasetService,
        private sidenavService: SideNavService,
        private router: Router,
        private modalService: ModalService) {
        this._window = window;
    }


    public ngOnInit(): void {
        if (this.sidenav) {
            this.sidenavService.setSidenav(this.sidenav);
            this.checkWindowSize();
        }
        this.router.events
        .pipe(filter(event => event instanceof NavigationEnd))
        .subscribe((event: any) => {
            this.initDatasetViewByType();
        });
        this.initDatasetViewByType();

        this.initTableData();

        this.prepareLinageGraph();

        this.appDatasetService.onSearchDatasetInfoChanges.subscribe((info: DatasetInfoInterface) => {
            this.datasetInfo = info;
        });
        this.appDatasetService.onSearchChanges.subscribe((value: string) => {
            this.searchValue = value;
        });

        /* eslint-disable  @typescript-eslint/no-explicit-any */
        this.appDatasetService.onSearchDataChanges.subscribe((data: any[]) => {
            this.tableData.tableSource = data;
        });

        this.appDatasetService.onSearchMetadataChanges.subscribe((data: SearchOverviewInterface) => {
          this.tableData.tableSource = data.dataset;
          this.tableData.pageInfo = data.pageInfo;
          this.tableData.totalCount = data.totalCount;
          this.searchData = data.dataset;

          setTimeout(() => this.currentPage = data.currentPage);

        });
    }

    public changeLinageGraphView(): void {

        if (this.datasetViewType === DatasetViewTypeEnum.linage) {

            setTimeout(() => {
                const searchResultContainer: HTMLElement | null = document.getElementById('searchResultContainerContent');
                if (searchResultContainer !== null) {

                    const styleElement: CSSStyleDeclaration = getComputedStyle(searchResultContainer);
                    this.linageGraphView[0] = searchResultContainer.offsetWidth
                        - parseInt(styleElement.paddingLeft)
                        - parseInt(styleElement.paddingRight);
                    this.linageGraphView[1] = 400;
                }
            });
        }
    }

    public getDatasetTree(): { id: string, kind: DatasetKindTypeNames }[][] {
        return this.appDatasetService.getDatasetTree;
    }

    public ngAfterContentInit(): void {
        this.tableData.tableSource = this.searchData;
    }

    public onPageChange(params: {currentPage: number, isClick: boolean}): void {
        this.currentPage = params.currentPage;
        this.initDatasetViewByType(params.currentPage);
    }


    public getResultUnitText(): string {
        const searchDataset: string = this.getDatasetId();
        return `results in ${searchDataset}`;
    }


    public momentConverDatetoLocalWithFormat(date: string): string {
        return AppValues.momentConverDatetoLocalWithFormat({
            date: new Date(String(date)),
            format: 'DD MMM YYYY',
            isTextDate: true
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
        this.modalService.warning({
          message: 'Feature will be soon',
          yesButtonText: 'Ok'
        });
    }

    public get datasetViewTypeOverview(): boolean {
        return this.datasetViewType === DatasetViewTypeEnum.overview;
    }

    public get datasetViewTypeMetadata(): boolean {
        return this.datasetViewType === DatasetViewTypeEnum.metadata;
    }

    public get datasetViewTypeLinage(): boolean {
        return this.datasetViewType === DatasetViewTypeEnum.linage;
    }

    public onSearchMetadata(currentPage: number): void {
        this.router.navigate([AppValues.defaultUsername, AppValues.urlDatasetView], {
            queryParams: {
                id: this.getDatasetId(),
                type: AppValues.urlDatasetViewMetadataType,
                p: currentPage
            }
        });
        this.currentPage = currentPage;

        this.datasetViewType = DatasetViewTypeEnum.metadata;
        this.appDatasetService.onSearchMetadata(this.getDatasetId(), currentPage - 1);
    }

    public onSearchDataset(page = 0): void {
        this.router.navigate([AppValues.defaultUsername, AppValues.urlDatasetView], {
            queryParams: {
                id: this.getDatasetId(),
                type: AppValues.urlDatasetViewOverviewType
            }
        });

        this.datasetViewType = DatasetViewTypeEnum.overview;

        this.appDatasetService.searchDataset(this.getDatasetId(), page);
    }

    public onSearchLinageDataset(): void {
        this.router.navigate([AppValues.defaultUsername, AppValues.urlDatasetView], {
            queryParams: {
                id: this.getDatasetId(),
                type: DatasetViewTypeEnum.linage
            }
        });

        this.datasetViewType = DatasetViewTypeEnum.linage;
        this.appDatasetService.resetDatasetTree();
        this.appDatasetService.onSearchLinageDataset(this.getDatasetId());

        this.changeLinageGraphView();
    }

    public onSearchProjections(): void {
        console.log('Projections Tab');
        this.onSearchDataset();
    }

    public onClickNode(idDataset: string): void {
        this.datasetViewType = DatasetViewTypeEnum.overview;
        this.onSelectDataset(idDataset);
    }

    private initLinageGraphProperty(): void {
        this.linageGraphNodes = [];
        this.linageGraphLink = [];
    }

    private prepareLinageGraph(): void {
        this.appDatasetService.resetDatasetTree();
        this.appDatasetService.resetKindInfo();
        this.initLinageGraphProperty();
        this.linageGraphClusters = [{
            id: DatasetKindTypeNames.root + '_cluster',
            label: DatasetKindTypeNames.root,
            data: {customColor: '#A52A2A59'},
            position: {x: 10, y: 10},
            childNodeIds: []
        }, {
            id: DatasetKindTypeNames.derivative + '_cluster',
            label: DatasetKindTypeNames.derivative,
            data: {customColor: '#00800039'},
            position: {x: 10, y: 10},
            childNodeIds: []
        }];

        let uniqDatasetIdList: string[] = [];

        this.appDatasetService.onDatasetTreeChanges.subscribe((datasetTree: {id: string, kind: DatasetKindTypeNames}[][]) => {
            this.isAvailableLinageGraph = (datasetTree.length !== 0);
            datasetTree.forEach((term: {id: string, kind: DatasetKindTypeNames}[]) => term.forEach((termInfo: {id: string, kind: DatasetKindTypeNames}) => uniqDatasetIdList.push(termInfo.id)));
            uniqDatasetIdList = uniqDatasetIdList.filter((x: any, y: number) => uniqDatasetIdList.indexOf(x) === y);

            this.initLinageGraphProperty();

            if (datasetTree.length) {
            datasetTree.forEach((term: {id: string, kind: DatasetKindTypeNames}[], index: number) => {
                let source: string = term[0].id;
                let target: string = term[1].id;
                term.forEach((termInfo: {id: string, kind: DatasetKindTypeNames}) => {
                    if (termInfo.kind === DatasetKindTypeNames.root) {
                        source = termInfo.id;
                    } else {
                        target = termInfo.id;
                    }
                });

                this.linageGraphLink.push({
                    id: `${source}__and__${target}`,
                    source,
                    target,
                    label: `${source}__and__${target}`,
                });
            });

            uniqDatasetIdList.forEach((id: string) => {
                const oneOfTheKindInfo: DatasetKindInterface[] =
                    this.appDatasetService.kindInfo.filter((dataset: DatasetKindInterface) => dataset.id === id);


                this.linageGraphNodes.push({
                    id,
                    label: id,
                    data: {
                        kind: oneOfTheKindInfo[0] && oneOfTheKindInfo[0].kind,
                        customColor: oneOfTheKindInfo[0] && oneOfTheKindInfo[0].kind === DatasetKindTypeNames.root ? 'rgba(165,42,42,0.35)' : '#008000'}
                });
            });

            if (this.linageGraphNodes.length >= 1) {
                const linageGraphAllNodes: Node[] = this.linageGraphNodes.filter((n: Node) => n.data.kind === DatasetKindTypeNames.root);
                const linageGraphDerivativeNodes: Node[] = this.linageGraphNodes.filter((n: Node) => n.data.kind === DatasetKindTypeNames.derivative);
                linageGraphDerivativeNodes.forEach((n: Node) => {
                    linageGraphAllNodes.push(n);
                });

                linageGraphAllNodes.forEach((n: Node, index: number) => {
                    n.id = String(index);
                    this.linageGraphLink.forEach((e: Edge) => {
                        if (e.source === n.label) {
                            e.source = n.id;
                        }
                        if (e.target === n.label) {
                            e.target = n.id;
                        }
                    });
                });

                this.linageGraphNodes = linageGraphAllNodes;
            }
            }
        });

        this.appDatasetService.onKindInfoChanges.subscribe((datasetList: DatasetKindInterface[]) => {
            datasetList.forEach((dataset: DatasetKindInterface) => {
                this.linageGraphClusters = this.linageGraphClusters.map((cluster: ClusterNode) => {
                    if (typeof cluster.childNodeIds === 'undefined') {
                        cluster.childNodeIds = [];
                    }

                    if (cluster.label === dataset.kind) {
                        cluster.childNodeIds.push(dataset.id);
                    }
                    return cluster;
                });
            });
        });
    }

    private onClickDeriveForm(): void {
        console.log('onClickDeriveForm');
    }

    private onClickExplore(): void {
        console.log('onClickExplore');
    }

    private onClickReputation(): void {
        console.log('onClickReputation');
    }

    private onClickDescission(): void {
        console.log('onClickDescission');
    }

    private initTableData(): void {
        this.tableData = {
            isTableHeader: true,
            tableSource: this.searchData,
            isResultQuantity: true,
            isClickableRow: false,
            pageInfo: {
                hasNextPage: false,
                hasPreviousPage: false,
                totalPages: 1
              },
            totalCount: 0
        };
    }

    private initDatasetViewByType(currentPage?: number): void {
        if (this.appDatasetService.onSearchLinageDatasetSubscribtion) {
            this.appDatasetService.onSearchLinageDatasetSubscribtion.unsubscribe();
        }
        this.appDatasetService.resetDatasetTree();
        const searchParams: string[] = this._window.location.search.split('&type=');
        const searchPageParams: string[] = this._window.location.search.split('&p=');
        let page = 1;
        if (searchPageParams[1]) {
            page = currentPage || Number(searchPageParams[1].split('&')[0]);
        }

        if (searchParams.length > 1) {
            const type: DatasetViewTypeEnum = AppValues.fixedEncodeURIComponent(searchParams[1].split('&')[0]) as DatasetViewTypeEnum;

            this.datasetViewType = type;
            if (type === DatasetViewTypeEnum.overview) {
                this.onSearchDataset();
            }
            if (type === DatasetViewTypeEnum.metadata) {
                this.currentPage = page;
                this.onSearchMetadata(page);
            }
            if (type === DatasetViewTypeEnum.linage) {
                this.onSearchLinageDataset();
            }
            if (type === DatasetViewTypeEnum.projections) {
                this.onSearchProjections();
            }
        }
    }

    private getDatasetId(): string {
        const searchParams: string[] = this._window.location.search.split('?id=');

        if (searchParams.length > 1) {
            return AppValues.fixedEncodeURIComponent(searchParams[1].split('&')[0]);
        }
        return '';
    }

    public onSelectDataset(id: string): void {
        this.router.navigate([AppValues.defaultUsername, AppValues.urlDatasetView], {
            queryParams: {
                id,
                type: AppValues.urlDatasetViewOverviewType
            }
        });
    }
    ngOnDestroy() {
        if (this.appDatasetService.onSearchLinageDatasetSubscribtion) {
            this.appDatasetService.onSearchLinageDatasetSubscribtion.unsubscribe();
        }
    }
}
